use crate::{
    m20260318_233726_group_table::Group, m20260323_002518_zettel_table::Zettel,
    m20260327_175853_tag_table::Tag, types::NANO_ID_LEN,
};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Group::Table).to_owned())
            .await?;

        // just recreate the groups table!
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .col(pk_auto(Group::Id))
                    .col(
                        string(Group::NanoId)
                            .string_len(NANO_ID_LEN as u32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string(Group::Name).not_null())
                    .col(string(Group::Priority).not_null())
                    .col(date_time(Group::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(Group::ModifiedAt).default(Expr::current_timestamp()))
                    .col(
                        string(Group::ZettelId)
                            .string_len(NANO_ID_LEN as u32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string_null(Group::ParentGroupId).string_len(NANO_ID_LEN as u32))
                    .col(
                        string(Group::TagId)
                            .string_len(NANO_ID_LEN as u32)
                            .unique_key()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_zettel_id")
                            .from(Group::Table, Group::ZettelId)
                            .to(Zettel::Table, Zettel::NanoId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_parent_id")
                            .from(Group::Table, Group::ParentGroupId)
                            .to(Group::Table, Group::NanoId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_tag_id")
                            .from(Group::Table, Group::TagId)
                            .to(Tag::Table, Tag::NanoId)
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
            .await?;

        // just create the shitty old table
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .col(pk_auto(Group::Id))
                    .col(
                        string(Group::NanoId)
                            .string_len(NANO_ID_LEN as u32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string(Group::Name).not_null())
                    .col(integer(Group::Color).not_null())
                    .col(string(Group::Priority).not_null())
                    .col(date_time(Group::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(Group::ModifiedAt).default(Expr::current_timestamp()))
                    .col(string(Group::ZettelId).not_null().unique_key())
                    .col(string_null(Group::ParentGroupId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_zettel_id")
                            .from(Group::Table, Group::ZettelId)
                            .to(Zettel::Table, Zettel::NanoId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_parent_id")
                            .from(Group::Table, Group::ParentGroupId)
                            .to(Group::Table, Group::NanoId)
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
}
