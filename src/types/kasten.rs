use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use color_eyre::eyre::{Context, Result};
use dto::{Database, DatabaseConnection, Migrator, MigratorTrait};
use tokio::{
    fs::{File, create_dir_all},
    sync::RwLock,
};
use tracing::debug;

use crate::types::{FrontMatter, Index, ZettelId, index::ZettelOnDisk};

#[derive(Debug, Clone)]
pub struct Kasten {
    /// Private field so it can only be instantiated from a `Path`
    _private: (),

    pub root: PathBuf,

    pub index: Index,

    pub db: DatabaseConnection,
}

pub type KastenHandle = Arc<RwLock<Kasten>>;

impl Kasten {
    /// Given a path, try to construct a `Kasten` based on its contents.
    ///
    /// Note: this means that there should already exist a valid `Kasten`
    /// at that path.
    pub async fn instansiate(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        let db_conn_string = format!(
            "sqlite://{}",
            root.clone()
                .join(".filaments/filaments.db")
                .canonicalize()
                .context("Invalid Filaments workspace!!")?
                .to_string_lossy()
        );

        debug!("connecting to {db_conn_string}");

        let conn = Database::connect(db_conn_string)
            .await
            .context("Failed to connect to the database in the filaments workspace!")?;

        let index = Index::tabulate(&root)?;

        // run da migrations every time we connect, just in case
        Migrator::up(&conn, None).await?;

        Ok(Self {
            _private: (),
            db: conn,
            root,
            index,
        })
    }

    /// Create a new `Kasten` at the provided `path`.
    pub async fn initialize(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        let filaments_dir = path.join(".filaments");

        // create the dir
        create_dir_all(&filaments_dir)
            .await
            .context("Failed to create the filaments directory!")?;

        let filaments_dir = filaments_dir.canonicalize()?;

        File::create(filaments_dir.join("filaments.db")).await?;

        Ok(Self::instansiate(&path).await.expect(
            "Invariant broken. This instantiation call must always work \
         since we just initialized the workspace.",
        ))
    }

    /// processes the `Zettel` for the provided `ZettelId`,
    /// meaning it updates the internal state of the `Kasten`
    /// with the changes in `Zettel`.
    pub async fn process_path(&mut self, path: impl AsRef<Path>) -> Result<()> {
        //NOTE: need to clone to get around borrowing rules but
        // ideally we dont have to do this, kind of cringe imo.

        let path = path.as_ref().canonicalize()?;
        let zid = ZettelId::try_from(path.as_path())?;

        if !self.index.zods.contains_key(&zid) {
            let (fm, body) = FrontMatter::extract_from_file(&path)?;
            self.index.zods.insert(
                zid.clone(),
                ZettelOnDisk {
                    fm,
                    body,
                    path: path.clone(),
                },
            );
        }

        // incase the path of the zettel changed
        self.index.update_path_for_zid(&zid, path.clone());
        // let the index process the zettel, basically update the internal state of the zod
        self.index.process_zid(&zid)?;
        // and then we sync tags
        self.index.sync_tags_with_db(&zid, &self.db).await?;
        self.index.sync_title_with_db(&zid, &self.db).await?;

        Ok(())
    }
}
