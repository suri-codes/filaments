use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20260323_002518_zettel_table::Zettel, m20260327_175853_tag_table::Tag};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ZettelTag::Table)
                    .if_not_exists()
                    .col(string(ZettelTag::ZettelNanoId).not_null())
                    .col(string(ZettelTag::TagNanoId).not_null())
                    .primary_key(
                        Index::create()
                            .col(ZettelTag::ZettelNanoId)
                            .col(ZettelTag::TagNanoId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-zettel_tag-zettel_nano_id")
                            .from(ZettelTag::Table, ZettelTag::ZettelNanoId)
                            .to(Zettel::Table, Zettel::NanoId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-zettel_tag-tag_nano_id")
                            .from(ZettelTag::Table, ZettelTag::TagNanoId)
                            .to(Tag::Table, Tag::NanoId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ZettelTag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ZettelTag {
    Table,
    ZettelNanoId,
    TagNanoId,
}
