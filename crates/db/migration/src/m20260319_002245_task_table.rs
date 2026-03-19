use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260318_233726_group_table::Group;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(pk_auto(Task::Id))
                    .col(string(Task::NanoId).unique_key().not_null())
                    .col(string(Task::Name).not_null())
                    .col(string(Task::DescriptionPath).not_null())
                    .col(integer(Task::Priority).not_null().default(0))
                    .col(timestamp(Task::Due).null())
                    .col(timestamp(Task::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Task::ModifiedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_group_id") // unique constraint name
                            .from(Task::Table, Task::GroupId)
                            .to(Group::Table, Group::NanoId) // self-referential to the nano-id
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_pub_id")
                    .table(Task::Table)
                    .col(Task::NanoId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_group_id")
                    .table(Task::Table)
                    .col(Task::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_due")
                    .table(Task::Table)
                    .col(Task::Due)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_tasks_due").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_group_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_pub_id").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,

    /// Unique integer id
    Id,

    /// Unique nano-id that is userfacing
    NanoId,

    /// Nano-id of the group this task is a part of
    GroupId,

    /// Name of the Task
    Name,

    /// Priority level of the group
    Priority,

    /// The relative file path to the location of
    /// the description note for this task
    DescriptionPath,

    /// The duedate for this task
    Due,

    /// Creation time
    CreatedAt,

    /// Last modified
    ModifiedAt,
}
