use dto::{NanoId, TagModel, TagModelEx};

use crate::types::Color;

/// Represents a `Tag` in a `ZettelKasten` note taking method.
/// Easy way to link multiple notes under one simple word.
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Tag {
    /// Should only be constructed from models.
    _private: (),

    /// A unique `NanoId`
    pub id: NanoId,
    /// Name of the tag
    pub name: String,
    /// Color of the tag
    pub color: Color,
}

impl From<TagModel> for Tag {
    fn from(value: TagModel) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            color: value.color.into(),
        }
    }
}

impl From<TagModelEx> for Tag {
    fn from(value: TagModelEx) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            color: value.color.into(),
        }
    }
}
