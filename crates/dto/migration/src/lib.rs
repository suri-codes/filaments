pub use sea_orm_migration::prelude::*;

pub mod types;

mod m20260318_233726_group_table;
mod m20260319_002245_task_table;
mod m20260323_002518_zettel_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260318_233726_group_table::Migration),
            Box::new(m20260319_002245_task_table::Migration),
            Box::new(m20260323_002518_zettel_table::Migration),
        ]
    }
}
