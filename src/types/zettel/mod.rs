use std::path::{Path, PathBuf};

use dto::{
    DatabaseConnection, DateTime, TagEntity, ZettelActiveModel, ZettelEntity, ZettelModelEx,
};

use color_eyre::eyre::Result;
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

    pub async fn new(title: impl Into<String>, kt: &mut Kasten) -> Result<Self> {
        // fn new(title: impl Into<String>) -> Result<Self> {
        let title = title.into();

        // make a file that has a random identifier, and then
        // also has the name "title"
        let nano_id = NanoId::default();

        let local_file_path = format!("{nano_id}.md");

        // now we have to create the file
        let mut file = File::create_new(kt.root.clone().join(&local_file_path)).await?;

        let inserted = ZettelActiveModel::builder()
            .set_title(title.clone())
            .set_file_path(local_file_path)
            .set_nano_id(nano_id)
            .insert(&kt.db)
            .await?;

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

        kt.process_path(zettel.file_path.clone()).await?;

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
