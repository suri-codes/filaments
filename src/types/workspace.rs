use std::path::PathBuf;

use color_eyre::eyre::{Context, Result};
use dto::{Database, DatabaseConnection, Migrator, MigratorTrait};
use tokio::fs::{File, create_dir_all};
use tracing::debug;

/// The `Workspace` in which the filaments exist.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Private field so it can only be instantiated from a `Path`
    _private: (),
    /// Connection to the sqlite database inside the `Workspace`
    pub db: DatabaseConnection,
    /// The path to the root of this workspace
    pub root: PathBuf,
}

impl Workspace {
    /// Given a path, try to construct a `Workspace` based on its contents.
    ///
    /// Note: this means that there should already exist a valid `Workspace`
    /// at that path.
    pub async fn instansiate(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        let db_conn_string = format!(
            "sqlite://{}",
            path.clone()
                .join(".filaments/filaments.db")
                .canonicalize()
                .context("Invalid Filaments workspace!!")?
                .to_string_lossy()
        );

        debug!("connecting to {db_conn_string}");

        let conn = Database::connect(db_conn_string)
            .await
            .context("Failed to connect to the database in the filaments workspace!")?;

        // run da migrations every time we connect, just in case
        Migrator::up(&conn, None).await?;

        Ok(Self {
            _private: (),
            db: conn,
            root: path,
        })
    }

    pub async fn initialize(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        // create the .filaments folder
        let filaments_dir = path.join(".filaments");

        create_dir_all(&filaments_dir)
            .await
            .context("Failed to create the filaments directory!")?;

        // create the database inside there
        File::create(filaments_dir.join("filaments.db")).await?;

        Ok(Self::instansiate(path).await.expect(
            "Invariant broken. This instantiation call must always work
                    since we just initialized the workspace.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, create_dir_all},
        path::PathBuf,
    };

    use dto::NanoId;

    use crate::types::Workspace;

    #[tokio::test]
    async fn test_instantiation() {
        let path = PathBuf::from("/tmp/filaments/.filaments/filaments.db");
        create_dir_all(path.parent().unwrap()).unwrap();
        let _ = File::create(&path).unwrap();
        let _ws = Workspace::instansiate(dbg!(&path.parent().unwrap().parent().unwrap()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_initialization() {
        let path = PathBuf::from(format!("/tmp/filaments/{}", NanoId::default()));

        Workspace::initialize(path)
            .await
            .expect("Should initialize just fine");
    }
}
