use dto::{DateTime, GroupModelEx, NanoId};

use crate::types::{Color, Priority, Zettel};

/// A `Group` which contains tasks!
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Group {
    /// Should only be constructed from models.
    _private: (),

    pub id: NanoId,
    pub name: String,
    pub color: Color,
    pub priority: Priority,
    pub created_at: DateTime,
    pub modified_at: DateTime,
    /// The `Zettel` that is related to this `Group`.
    /// Can store notes regarding this group in
    /// the `Zettel`
    pub zettel: Zettel,
}

impl From<GroupModelEx> for Group {
    fn from(value: GroupModelEx) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            color: value.color.into(),
            priority: value.priority.into(),
            created_at: value.created_at,
            modified_at: value.modified_at,
            zettel: value
                .zettel
                .into_option()
                .expect(
                    "When fetching a Group from the database, we expect to always have the Zettel loaded!!",
                )
                .into(),
        }
    }
}
