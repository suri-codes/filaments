use std::path::Path;

use dto::{
    DatabaseConnection, DateTime, IntoActiveModel, TagEntity, ZettelActiveModel, ZettelEntity,
    ZettelModelEx,
};

use color_eyre::eyre::{Context, Result};
use dto::NanoId;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::types::{
    FrontMatter, Index, Kasten, Tag,
    frontmatter::{self, Body},
};

mod id;
pub use id::ZettelId;

/// A `Zettel` is a note about a single idea.
/// It can have many `Tag`s, just meaning it can fall under many
/// categories.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Zettel {
    /// Should only be constructed from models.
    _private: (),
    pub id: ZettelId,
    pub title: String,
    pub created_at: DateTime,
    pub modified_at: DateTime,
    pub tags: Vec<Tag>,
}

impl Zettel {
    /// fetches the `Zettel` with the provided `ZettelId`, returning `None` if not found.
    pub async fn fetch_from_db(zid: &ZettelId, db: &DatabaseConnection) -> Result<Option<Self>> {
        Ok(ZettelEntity::load()
            .filter_by_nano_id(zid.0.clone())
            .with(TagEntity)
            .one(db)
            .await?
            .map(Into::into))
    }

    pub async fn alter_name(
        id: ZettelId,
        new_name: impl Into<String>,
        kt: &mut Kasten,
    ) -> Result<()> {
        let new_name = new_name.into();

        // ok we need to change it on the actual zettel and then change
        // it in the frontmatter
        let _ = ZettelEntity::load()
            .filter_by_nano_id(id.clone())
            .one(&kt.db)
            .await?
            .expect("Must exist")
            .into_active_model()
            .set_title(new_name)
            .save(&kt.db)
            .await?;

        let zettel = Self::fetch_from_db(&id, &kt.db)
            .await?
            .expect("We just saved it");

        let file_path = zettel.absolute_path(&kt.index);
        let new_fm = FrontMatter::from(zettel);

        new_fm.flush_to_file(file_path)?;
        kt.index.process_zid(&id)?;

        Ok(())
    }

    pub async fn new(title: impl Into<String>, kt: &mut Kasten, tags: Vec<Tag>) -> Result<Self> {
        // fn new(title: impl Into<String>) -> Result<Self> {
        let title = title.into();

        // make a file that has a random identifier, and then
        // also has the name "title"
        let nano_id = NanoId::default();

        let local_file_path = format!("{nano_id}.md");

        let absolute_file_path = kt.root.clone().join(&local_file_path);

        // now we have to create the file
        let mut file = File::create_new(&absolute_file_path)
            .await
            .with_context(|| {
                format!("Failed to create file at local file path: {local_file_path}")
            })?;

        let inserted = {
            let mut am = ZettelActiveModel::builder()
                .set_title(title.clone())
                .set_nano_id(nano_id);

            for tag in tags {
                let tag = TagEntity::load()
                    .filter_by_nano_id(tag.id)
                    .one(&kt.db)
                    .await?
                    .expect("Invariant broken, tag must exist");
                am = am.add_tag(tag);
            }

            am.insert(&kt.db).await?
        };

        // need to load tags...
        let zettel = ZettelEntity::load()
            .filter_by_nano_id(inserted.nano_id)
            .with(TagEntity)
            .one(&kt.db)
            .await?
            .expect("This must exist since we just inserted it");

        let front_matter = FrontMatter::new(
            title,
            zettel.created_at,
            zettel.tags.iter().map(|t| t.name.clone()).collect(),
        );

        file.write_all(front_matter.to_string().as_bytes()).await?;

        kt.process_path(&absolute_file_path)
            .await
            .with_context(|| {
                format!(
                    "Kasten fails to process new Zettel at path: {}",
                    absolute_file_path.display(),
                )
            })?;

        Ok(zettel.into())
    }

    /// Returns the most up-to-date `FrontMatter` for this
    /// `Zettel`
    pub fn front_matter<'index>(&self, idx: &'index Index) -> &'index FrontMatter {
        &idx.get_zod(&self.id).fm
    }

    /// Returns the content of this `Zettel`, which is everything
    /// but the `FrontMatter`
    pub fn content<'index>(&self, idx: &'index Index) -> &'index Body {
        &idx.get_zod(&self.id).body
    }
    /// Get the absolute path to this `Zettel`
    pub fn absolute_path<'index>(&self, idx: &'index Index) -> &'index Path {
        &idx.get_zod(&self.id).path
    }

    /// Get the formatted creation datetime for this `Zettel`
    pub fn created_at(&self) -> String {
        self.created_at
            .format(frontmatter::DATE_FMT_STR)
            .to_string()
    }

    /// Get the formatted modified datetime for this `Zettel`
    pub fn modified_at(&self) -> String {
        self.modified_at
            .format(frontmatter::DATE_FMT_STR)
            .to_string()
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
            created_at: value.created_at,
            modified_at: value.modified_at,
            tags: value.tags.into_iter().map(Into::into).collect(),
        }
    }
}
