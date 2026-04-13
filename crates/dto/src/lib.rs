//! The DTO's (Data Transfer Objects) used to interact with
//! the Database. There is also a simple database struct in here.

/// For database stuff
pub use migration::{Migrator, MigratorTrait};
pub use sea_orm::{Database, DatabaseConnection};

/// exported traits for the database
pub use sea_orm::ActiveModelTrait;
pub use sea_orm::ActiveValue;
pub use sea_orm::ColumnTrait;
pub use sea_orm::EntityTrait;
pub use sea_orm::IntoActiveModel;
pub use sea_orm::QueryFilter;
pub use sea_orm::QueryOrder;
pub use sea_orm::entity::compound::HasMany;
pub use sea_orm::entity::compound::HasOne;

/// Exporting this as a generic NanoId.
pub use migration::types::NanoId;
/// Exporting this as DTO so we can newtype this in a later crate
/// and add additional functionality to it.
pub use migration::types::Priority as PriorityDTO;

pub use sea_orm::entity::prelude::Date;
pub use sea_orm::entity::prelude::DateTime;
pub use sea_orm::entity::prelude::Time;

/// Color type, exporting as DTO because I might
/// want to newtype wrap this, might not have to, depending
/// on how I end up using it in the application.
pub use migration::types::Color as ColorDTO;

mod entity;

/// Everything related to groups.
pub use entity::group::ActiveModel as GroupActiveModel;
pub use entity::group::ActiveModelEx as GroupActiveModelEx;
pub use entity::group::Column as GroupColumns;
pub use entity::group::Entity as GroupEntity;
pub use entity::group::Model as GroupModel;
pub use entity::group::ModelEx as GroupModelEx;

/// Everything related to tasks.
pub use entity::task::ActiveModel as TaskActiveModel;
pub use entity::task::ActiveModelEx as TaskActiveModelEx;
pub use entity::task::Column as TaskColumns;
pub use entity::task::Entity as TaskEntity;
pub use entity::task::Model as TaskModel;
pub use entity::task::ModelEx as TaskModelEx;

/// Everything related to zetetl's.
pub use entity::zettel::ActiveModel as ZettelActiveModel;
pub use entity::zettel::ActiveModelEx as ZettelActiveModelEx;
pub use entity::zettel::Column as ZettelColumns;
pub use entity::zettel::Entity as ZettelEntity;
pub use entity::zettel::Model as ZettelModel;
pub use entity::zettel::ModelEx as ZettelModelEx;

/// Everything related to tag's.
pub use entity::tag::ActiveModel as TagActiveModel;
pub use entity::tag::ActiveModelEx as TagActiveModelEx;
pub use entity::tag::Column as TagColumns;
pub use entity::tag::Entity as TagEntity;
pub use entity::tag::Model as TagModel;
pub use entity::tag::ModelEx as TagModelEx;

/// Everything related to the  zettel_tag entries.
pub use entity::zettel_tag::ActiveModel as ZettelTagActiveModel;
pub use entity::zettel_tag::ActiveModelEx as ZettelTagActiveModelEx;
pub use entity::zettel_tag::Column as ZettelTagColumns;
pub use entity::zettel_tag::Entity as ZettelTagEntity;
pub use entity::zettel_tag::Model as ZettelTagModel;
pub use entity::zettel_tag::ModelEx as ZettelTagModelEx;
