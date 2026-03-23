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
                    .table(Zettel::Table)
                    .if_not_exists()
                    .col(pk_auto(Zettel::Id))
                    .col(
                        string(Zettel::NanoId)
                            .string_len(NANO_ID_LEN as u32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string(Zettel::Title).not_null())
                    .col(string(Zettel::FilePath).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_zettel_pub_id")
                    .table(Zettel::Table)
                    .col(Zettel::NanoId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_zettel_pub_id").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("post").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Zettel {
    Table,

    /// Unique integer id
    Id,

    /// Unique nano-id that is user-facing
    NanoId,

    /// The title of this zettel
    Title,

    /// The relative file-path to this `Zettel`
    FilePath,
}
