use std::path::PathBuf;

use dto::{NanoId, ZettelModelEx};

use crate::types::Tag;

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
