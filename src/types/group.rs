use dto::{DateTime, GroupModelEx, NanoId};

use crate::types::{Priority, Tag, Zettel};

/// A `Group` which contains tasks!
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group {
    /// Should only be constructed from models.
    _private: (),

    pub id: NanoId,
    pub name: String,
    pub priority: Priority,
    pub parent_id: Option<NanoId>,
    pub created_at: DateTime,
    pub modified_at: DateTime,
    /// The `Zettel` that is related to this `Group`.
    /// Can store notes regarding this group in
    /// the `Zettel`
    pub zettel: Zettel,

    /// The `Tag` that is related to this `Group`
    pub tag: Tag,
}

impl From<GroupModelEx> for Group {
    fn from(value: GroupModelEx) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            priority: value.priority.into(),
            parent_id: value.parent_group_id,
            created_at: value.created_at,
            modified_at: value.modified_at,
            zettel: value
                .zettel
                .into_option()
                .expect(
                    "When fetching a Group from the database, we expect to always have the Zettel loaded!!",
                )
                .into(),
            tag: value
            .tag
            .into_option()
            .expect(
                "When fetching a Group from the database, we expect to always have the Zettel loaded!!",
            )
            .into(),
        }
    }
}
