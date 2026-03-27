use sea_orm_migration::{prelude::*, schema::*};

use crate::types::NANO_ID_LEN;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(pk_auto(Tag::Id))
                    .col(
                        string(Tag::NanoId)
                            .string_len(NANO_ID_LEN as u32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string(Tag::Name).not_null())
                    .col(integer(Tag::Color).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,

    /// Unique integer id
    Id,

    /// Unique userfacing nano-id
    NanoId,

    /// Name of the tag (case sensitive)
    Name,

    /// Color of this tag
    Color,
}
