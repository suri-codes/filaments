//! The database abstraction for the different actions `Filaments` requires
//! from a database service.

use std::path::Path;

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tracing::debug;

use crate::errors::{DbError, DbResult};

/// Database Errors
mod errors;
#[expect(unused_imports)]
pub use errors::*;

/// Database struct
#[expect(dead_code)]
#[derive(Debug)]
struct Db {
    conn: DatabaseConnection,
}

impl Db {
    async fn connect(path: &Path) -> DbResult<Self> {
        let connection_string = dbg! {format!(
            "sqlite://{}",
            path.canonicalize()
                .map_err(|_| DbError::NotFound {
                    not_found_at: path.to_string_lossy().to_string()
                })?
                .to_string_lossy()
        )};

        debug!("connecting to {connection_string}");

        let conn = Database::connect(connection_string).await?;

        // run all migrations on connection
        Migrator::up(&conn, None).await?;

        Ok(Self { conn })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, create_dir_all},
        path::PathBuf,
    };

    use crate::Db;

    #[tokio::test]
    async fn test_connect() {
        let path = PathBuf::new();
        let _ = Db::connect(&path).await.expect_err("not found");

        let path = PathBuf::from("/tmp/filaments/test_db.sqlite");
        create_dir_all(path.parent().unwrap()).unwrap();
        let _ = File::create(&path).unwrap();
        let _db = Db::connect(&path).await.unwrap();
    }
}
