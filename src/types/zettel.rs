use dto::{DateTime, TagEntity, ZettelActiveModel, ZettelEntity, ZettelModelEx};
use std::path::PathBuf;

use color_eyre::eyre::Result;
use dto::NanoId;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::types::{FrontMatter, Tag, Workspace};

/// A `Zettel` is a note about a single idea.
/// It can have many `Tag`s, just meaning it can fall under many
/// categories.
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Zettel {
    /// Should only be constructed from models.
    _private: (),
    pub id: NanoId,
    pub title: String,
    /// a workspace-local file path, needs to be canonicalized before usage
    pub file_path: PathBuf,
    pub created_at: DateTime,
    pub tags: Vec<Tag>,
}

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

    /// Returns the most up-to-date `FrontMatter` for this
    /// `Zettel`
    #[expect(dead_code)]
    pub async fn front_matter(&self, ws: &Workspace) -> Result<FrontMatter> {
        let path = self.absolute_path(ws);
        let (fm, _) = FrontMatter::extract_from_file(path).await?;
        Ok(fm)
    }

    /// Returns the content of this `Zettel`, which is everything
    /// but the `FrontMatter`
    #[expect(dead_code)]
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

    fn absolute_path(&self, ws: &Workspace) -> PathBuf {
        ws.root.clone().join(&self.file_path)
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
            id: value.nano_id,
            title: value.title,
            file_path: value.file_path.into(),
            created_at: value.created_at,
            tags: value.tags.into_iter().map(Into::into).collect(),
        }
    }
}
