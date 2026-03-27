//! The DTO's (Data Transfer Objects) used to interact with
//! the Database. There is also a simple database struct in here.

/// Database and its Errors
mod db;
pub use db::*;

/// exported traits for the database
pub use sea_orm::ActiveModelTrait;
pub use sea_orm::ActiveValue;

/// Exporting this as a generic NanoId.
pub use migration::types::NanoId;
/// Exporting this as DTO so we can newtype this in a later crate
/// and add additional functionality to it.
pub use migration::types::Priority as PriorityDTO;

mod entity;

/// Everything related to groups.
pub use entity::group::ActiveModel as GroupActiveModel;
pub use entity::group::ActiveModelEx as GroupActiveModelEx;
pub use entity::group::Entity as GroupEntity;
pub use entity::group::Model as GroupModel;
pub use entity::group::ModelEx as GroupModelEx;

/// Everything related to tasks.
pub use entity::task::ActiveModel as TaskActiveModel;
pub use entity::task::ActiveModelEx as TaskActiveModelEx;
pub use entity::task::Entity as TaskEntity;
pub use entity::task::Model as TaskModel;
pub use entity::task::ModelEx as TaskModelEx;

/// Everything related to zetetl's.
pub use entity::zettel::ActiveModel as ZettelActiveModel;
pub use entity::zettel::ActiveModelEx as ZettelActiveModelEx;
pub use entity::zettel::Entity as ZettelEntity;
pub use entity::zettel::Model as ZettelModel;
pub use entity::zettel::Model as ZettelModelEx;
