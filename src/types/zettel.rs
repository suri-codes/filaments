use dto::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DateTime, EntityTrait as _, IntoActiveModel,
    QueryFilter, TagActiveModel, TagEntity, ZettelActiveModel, ZettelActiveModelEx, ZettelEntity,
    ZettelModelEx, ZettelTagActiveModel, ZettelTagColumns, ZettelTagEntity,
};
use pulldown_cmark::{Event, Parser, Tag as MkTag};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use tracing::{error, info};

use color_eyre::eyre::{Error, Result, eyre};
use dto::NanoId;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::types::{FrontMatter, Kasten, Link, Tag, Workspace, frontmatter};

/// A `Zettel` is a note about a single idea.
/// It can have many `Tag`s, just meaning it can fall under many
/// categories.
#[derive(Debug, Clone)]
pub struct Zettel {
    /// Should only be constructed from models.
    _private: (),
    pub id: ZettelId,
    pub title: String,
    /// a workspace-local file path, needs to be canonicalized before usage
    pub file_path: PathBuf,
    pub created_at: DateTime,
    pub modified_at: DateTime,
    pub tags: Vec<Tag>,
}

/// A `ZettelId` is essentially a `NanoId`,
/// with some `Zettel` specific helpers written
/// onto it
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZettelId(NanoId);

impl Zettel {
    pub async fn new(title: impl Into<String>, ws: &Workspace) -> Result<Self> {
        // fn new(title: impl Into<String>) -> Result<Self> {
        let title = title.into();

        // make a file that has a random identifier, and then
        // also has the name "title"
        let nano_id = NanoId::default();

        let local_file_path = format!("{nano_id}.md");

        // now we have to create the file
        let mut file = File::create_new(ws.root.clone().join(&local_file_path)).await?;

        let inserted = ZettelActiveModel::builder()
            .set_title(title.clone())
            .set_file_path(local_file_path)
            .set_nano_id(nano_id)
            .insert(&ws.db)
            .await?;

        // need to load tags...
        let zettel = ZettelEntity::load()
            .filter_by_nano_id(inserted.nano_id)
            .with(TagEntity)
            .one(&ws.db)
            .await?
            .expect("This must exist since we just inserted it");

        let front_matter = FrontMatter::new(
            title,
            zettel.created_at,
            zettel.tags.iter().map(|t| t.name.clone()).collect(),
        );

        file.write_all(front_matter.to_string().as_bytes()).await?;

        Ok(zettel.into())
    }

    pub async fn sync_with_file(&mut self, ws: &Workspace) -> Result<()> {
        let (fm, content) = FrontMatter::extract_from_file(self.absolute_path(ws)).await?;

        self.title = fm.title;

        // it could have new tags and stuff...

        // todo!();

        Ok(())
    }

    /// Returns the most up-to-date `FrontMatter` for this
    /// `Zettel`
    pub async fn front_matter(&self, ws: &Workspace) -> Result<FrontMatter> {
        let path = self.absolute_path(ws);
        let (fm, _) = FrontMatter::extract_from_file(path).await?;
        Ok(fm)
    }

    /// Returns the content of this `Zettel`, which is everything
    /// but the `FrontMatter`
    pub async fn content(&self, ws: &Workspace) -> Result<String> {
        let path = self.absolute_path(ws);
        let (_, content) = FrontMatter::extract_from_file(path).await?;
        Ok(content)
    }

    #[expect(dead_code)]
    async fn open_file(&self, ws: &Workspace) -> Result<File> {
        let path = self.absolute_path(ws);
        Ok(File::open(path).await?)
    }

    pub fn absolute_path(&self, ws: &Workspace) -> PathBuf {
        ws.root.clone().join(&self.file_path)
    }

    /// uses the id and root to parse out of the root directory
    #[expect(dead_code)]
    pub async fn from_id(id: &ZettelId, ws: &Workspace) -> Result<Self> {
        let mut path = ws.root.clone();
        path.push(id.0.to_string());
        Self::from_path(path, ws).await
    }

    pub fn created_at(&self) -> String {
        self.created_at
            .format(frontmatter::DATE_FMT_STR)
            .to_string()
    }

    pub fn modified_at(&self) -> String {
        self.modified_at
            .format(frontmatter::DATE_FMT_STR)
            .to_string()
    }

    pub async fn from_path(path: impl Into<PathBuf>, ws: &Workspace) -> Result<Self> {
        let path: PathBuf = path.into();

        let id = ZettelId::try_from(path.as_path())?;

        let (front_matter, _) = FrontMatter::extract_from_file(&ws.root.clone().join(path)).await?;

        let mut zettel_tag_strings = front_matter.tag_strings.clone();

        zettel_tag_strings.sort();

        // get the zettel from the db
        let db_zettel: ZettelModelEx = if let Some(existing_zettel) = ZettelEntity::load()
            .with(TagEntity)
            .filter_by_nano_id(id.clone())
            .one(&ws.db)
            .await?
        {
            existing_zettel
        } else {
            // if zettel is missing from db, we just add it here
            info!("adding zettel to db");
            let am = ZettelActiveModel {
                nano_id: ActiveValue::Set(id.clone().into()),
                title: ActiveValue::Set(front_matter.title.clone()),
                ..Default::default()
            };

            am.insert(&ws.db).await?;

            ZettelEntity::load()
                .with(TagEntity)
                .filter_by_nano_id(id.clone())
                .one(&ws.db)
                .await?
                .expect("we just inserted the zettel")
        };

        // get the tags for it
        for db_tag in db_zettel.tags {
            if let Ok(idx) = zettel_tag_strings.binary_search(&db_tag.name) {
                // we remove tags we have already processed
                zettel_tag_strings.remove(idx);
            } else {
                // the db says the file has tag `x`, but that tag is missing from the
                // front matter, we can assume its gone, lets delete that link
                let to_remove = ZettelTagEntity::find()
                    .filter(ZettelTagColumns::ZettelNanoId.eq(id.0.clone()))
                    .filter(ZettelTagColumns::TagNanoId.eq(db_tag.nano_id))
                    .one(&ws.db)
                    .await?
                    .expect("this link must exist");

                to_remove.into_active_model().delete(&ws.db).await?;
            }
        }

        // now any tags that are left inside zettel_tag_strings,
        // we have to put them inside the db
        for new_tag in zettel_tag_strings {
            // create a new tag
            let tag = TagActiveModel {
                name: ActiveValue::Set(new_tag),
                ..Default::default()
            }
            .insert(&ws.db)
            .await?;

            // this zettel has this tag now
            let _ = ZettelTagActiveModel {
                zettel_nano_id: ActiveValue::Set(id.to_string()),
                tag_nano_id: ActiveValue::Set(tag.nano_id.to_string()),
            }
            .insert(&ws.db)
            .await?;
        }

        if front_matter.title != db_zettel.title {
            let am = ZettelActiveModel {
                id: ActiveValue::Unchanged(db_zettel.id),
                title: ActiveValue::Set(front_matter.title.clone()),
                ..Default::default()
            };

            am.update(&ws.db).await?;
        }

        Ok(ZettelEntity::load()
            .with(TagEntity)
            .filter_by_nano_id(id.clone())
            .one(&ws.db)
            .await?
            .expect("We just inserted it right above")
            .into())
    }

    /// The `Link`s that are going out of this `Zettel`
    pub async fn links(&self, ws: &Workspace) -> Result<Vec<Link>> {
        let content = self.content(ws).await?;
        let parsed = Parser::new(&content);

        let mut links = vec![];

        for event in parsed {
            if let Event::Start(MkTag::Link { dest_url, .. }) = event {
                info!("Found dest_url: {dest_url:#?}");

                let dest_path = {
                    // remove leading "./"
                    let without_prefix = dest_url.strip_prefix("./").unwrap_or(&dest_url);

                    // remove "#" and everything after it
                    let without_anchor = without_prefix.split('#').next().unwrap();

                    // add .md if not present
                    let normalized = if std::path::Path::new(without_anchor)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                    {
                        without_anchor.to_string()
                    } else {
                        format!("{without_anchor}.md")
                    };

                    let mut tmp_root = ws.root.clone();
                    tmp_root.push(normalized);
                    tmp_root
                };
                // simplest way to validate that the path exists
                let Ok(canon_url) = dest_path.canonicalize() else {
                    error!("Link not found!: {dest_path:?}");
                    continue;
                };

                let dst_id = ZettelId::try_from(canon_url)?;

                let link = Link::new(self.id.clone(), dst_id);

                links.push(link);
            }
        }

        Ok(links)
    }
}

impl From<ZettelModelEx> for Zettel {
    fn from(value: ZettelModelEx) -> Self {
        assert!(
            !value.tags.is_unloaded(),
            "When fetching a Zettel from the database, we expect
            to always have the tags loaded!!"
        );

        Self {
            _private: (),
            id: value.nano_id.into(),
            title: value.title,
            file_path: value.file_path.into(),
            created_at: value.created_at,
            modified_at: value.modified_at,
            tags: value.tags.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<&str> for ZettelId {
    fn from(value: &str) -> Self {
        Self(NanoId::from(value))
    }
}

impl From<&NanoId> for ZettelId {
    fn from(value: &NanoId) -> Self {
        value.clone().into()
    }
}

impl From<NanoId> for ZettelId {
    fn from(value: NanoId) -> Self {
        Self(value)
    }
}

impl TryFrom<PathBuf> for ZettelId {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let path = value.as_path();
        path.try_into()
    }
}

impl TryFrom<&Path> for ZettelId {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let extension = value
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| eyre!("Unable to turn file extension into string".to_owned(),))?;

        if extension != "md" {
            return Err(eyre!(format!("Wrong extension: {extension}, expected .md")));
        }

        let id: Self = value
            .file_name()
            .ok_or_else(|| eyre!("Invalid File Name!".to_owned()))?
            .to_str()
            .ok_or_else(|| eyre!("File Name cannot be translated into str!".to_owned(),))?
            .strip_suffix(".md")
            .expect("we statically verify this right above")
            .into();

        Ok(id)
    }
}

impl Display for ZettelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl From<ZettelId> for NanoId {
    fn from(value: ZettelId) -> Self {
        value.0
    }
}
