use dto::{TagEntity, ZettelActiveModel, ZettelEntity, ZettelModelEx};
use std::path::PathBuf;

use color_eyre::eyre::Result;
use dto::NanoId;
use tokio::fs::File;

use crate::types::{Tag, Workspace};

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
        File::create_new(ws.root.clone().join(&local_file_path)).await?;

        let inserted = ZettelActiveModel::builder()
            .set_title(title)
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

        Ok(zettel.into())
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
            tags: value.tags.into_iter().map(Into::into).collect(),
        }
    }
}
