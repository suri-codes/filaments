use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260323_002518_zettel_table::Zettel;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Zettel::Table)
                    .drop_column(Zettel::FilePath)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Zettel::Table)
                    .add_column(string(Zettel::FilePath).not_null())
                    .to_owned(),
            )
            .await
    }
}
