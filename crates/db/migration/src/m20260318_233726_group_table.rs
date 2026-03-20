use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .if_not_exists()
                    .col(pk_auto(Group::Id))
                    .col(string(Group::NanoId).unique_key().not_null())
                    .col(string(Group::Name).not_null())
                    //Note: Color is a hex color with the leading #
                    .col(string(Group::Color).not_null())
                    .col(string(Group::DescriptionPath).not_null())
                    .col(integer(Group::Priority).not_null().default(0))
                    .col(timestamp(Group::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Group::ModifiedAt).default(Expr::current_timestamp()))
                    .col(string_null(Group::ParentGroupId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_parent_id") // unique constraint name
                            .from(Group::Table, Group::ParentGroupId)
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
                    .name("idx_groups_pub_id")
                    .table(Group::Table)
                    .col(Group::NanoId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_groups_pub_id").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Group::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Group {
    Table,

    /// Unique integer id
    Id,

    /// Unique nano-id that is userfacing
    NanoId,

    /// Nano-id of the parent of this group
    ParentGroupId,

    /// Name of the group
    Name,

    /// Color of this group
    /// NOTE: color is a string that looks like "#FFFFFF"
    Color,

    /// Priority level of the group
    Priority,

    /// The relative file path to the location of
    /// the description note for this task
    DescriptionPath,

    /// Creation time
    CreatedAt,

    /// Last modified
    ModifiedAt,
}
