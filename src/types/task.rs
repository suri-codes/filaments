use color_eyre::eyre::{Context, Result, eyre};
use dto::{
    DateTime, GroupEntity, HasOne, IntoActiveModel as _, NanoId, TagEntity, TaskActiveModel,
    TaskEntity, TaskModelEx, ZettelEntity,
};

use crate::types::{Group, Kasten, Priority, Zettel, frontmatter};

/// a `Task` that you have to complete!
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    /// Should only be constructed from models.
    _private: (),

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

impl Task {
    pub async fn new(
        name: impl Into<String>,
        parent_id: NanoId,
        kt: &mut Kasten,
        due: Option<DateTime>,
        priority: Priority,
    ) -> Result<Self> {
        let name = name.into();

        let parent = GroupEntity::load()
            .with(TagEntity)
            .filter_by_nano_id(parent_id)
            .one(&kt.db)
            .await
            .with_context(|| "failed to communicate with db")?
            .ok_or_else(|| eyre!("could not find the group"))?;

        let HasOne::Loaded(tag) = parent.tag else {
            panic!("this has to be loaded since we just loaded it right above")
        };

        let zettel = Zettel::new(name.clone(), kt, vec![(*tag).into()]).await?;

        let inserted = TaskActiveModel::builder()
            .set_name(name)
            .set_group_id(parent.nano_id.clone())
            .set_priority(priority)
            .set_zettel(
                ZettelEntity::load()
                    .filter_by_nano_id(zettel.id)
                    .one(&kt.db)
                    .await?
                    .expect("Zettel must exist since we just created it")
                    .into_active_model(),
            )
            .set_due(due)
            .insert(&kt.db)
            .await?;

        let group = GroupEntity::load()
            .with(TagEntity)
            .with((ZettelEntity, TagEntity))
            .filter_by_nano_id(parent.nano_id)
            .one(&kt.db)
            .await?
            .expect("We just inserted it");

        let mut task_am = TaskEntity::load()
            .with((ZettelEntity, TagEntity))
            .filter_by_nano_id(inserted.nano_id)
            .one(&kt.db)
            .await?
            .expect("We just inserted it");

        task_am.group = HasOne::Loaded(Box::new(group));

        let task: Self = task_am.into();

        kt.todo_tree.insert_task(&task);

        Ok(task)
    }

    pub async fn alter_name(
        id: NanoId,
        new_name: impl Into<String>,
        kt: &mut Kasten,
    ) -> Result<()> {
        let new_name = new_name.into();

        let task = TaskEntity::load()
            .filter_by_nano_id(id.clone())
            .one(&kt.db)
            .await?
            .expect("Invariant Broken: Must exist");

        let zettel_id = task.zettel_id.clone();

        let _ = task
            .into_active_model()
            .set_name(new_name.as_str())
            .save(&kt.db)
            .await?;

        Zettel::alter_name(zettel_id.into(), new_name, kt).await?;

        Ok(())
    }

    pub async fn alter_priority(id: NanoId, new_prio: Priority, kt: &Kasten) -> Result<()> {
        TaskEntity::load()
            .filter_by_nano_id(id)
            .one(&kt.db)
            .await?
            .expect("Must exist")
            .into_active_model()
            .set_priority(new_prio)
            .update(&kt.db)
            .await?;

        Ok(())
    }

    pub fn due(&self) -> Option<String> {
        self.due
            .map(|due| due.format(frontmatter::DATE_FMT_STR).to_string())
    }
    pub fn finished_at(&self) -> Option<String> {
        self.finished_at
            .map(|finished_at| finished_at.format(frontmatter::DATE_FMT_STR).to_string())
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
