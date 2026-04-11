use dto::{ DateTime, NanoId, TaskModelEx};

use crate::types::{Group, Priority, Zettel};

/// a `Task` that you have to complete!
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Task {
    /// Should only be constructed from models.
    _private:(),

    pub id: NanoId,
    pub name: String,
    pub priority: Priority,
    pub due: Option<DateTime>,
    pub group_id: NanoId,
    pub finished_at: Option<DateTime>,
    pub created_at: DateTime,
    pub modified_at: DateTime,
    /// Each task has its own related `Zettel`.
    pub zettel: Zettel,
    pub group: Group,
}

impl From<TaskModelEx> for Task {
    fn from(value: TaskModelEx) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            priority: value.priority.into(),
            due: value.due,
            group_id: value.group_id,
            finished_at: value.finished_at,
            created_at: value.created_at,
            modified_at: value.modified_at,
            zettel: value
                .zettel
                .into_option()
                .expect(
                    "When fetching a Task from the database, we expect to always have the Zettel loaded!!",
                )
                .into(),
            group: value
                .group
                .into_option()
                .expect(
                    "When fetching a Task from the database, we expect to always have the Group loaded!!",
                )
                .into(),
            
        }
    }
}
