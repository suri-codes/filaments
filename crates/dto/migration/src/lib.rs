pub use sea_orm_migration::prelude::*;

pub mod types;

mod m20260318_233726_group_table;
mod m20260319_002245_task_table;
mod m20260323_002518_zettel_table;
mod m20260327_175853_tag_table;
mod m20260327_180618_zettel_tag_table;
mod m20260406_200424_remove_path_from_zettel;
mod m20260410_223725_add_tag_to_group;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260318_233726_group_table::Migration),
            Box::new(m20260319_002245_task_table::Migration),
            Box::new(m20260323_002518_zettel_table::Migration),
            Box::new(m20260327_175853_tag_table::Migration),
            Box::new(m20260327_180618_zettel_tag_table::Migration),
            Box::new(m20260406_200424_remove_path_from_zettel::Migration),
            Box::new(m20260410_223725_add_tag_to_group::Migration),
        ]
    }
}
