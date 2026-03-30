use std::path::PathBuf;

use color_eyre::eyre::Result;

use crate::types::Workspace;

/// The `Kasten` that stores the `Link`s between `Zettel`s
#[derive(Debug, Clone)]
#[expect(dead_code)]
pub struct Kasten {
    /// Private field so it can only be instantiated from a `Path`
    _private: (),
    // / Connection to the sqlite database inside the `Workspace`
    pub ws: Workspace,
}

impl Kasten {
    /// Given a path, try to construct a `Kasten`.
    pub async fn index(root: impl Into<PathBuf>) -> Result<Self> {
        let ws = Workspace::instansiate(root).await?;

        Ok(Self { _private: (), ws })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, create_dir_all},
        path::PathBuf,
    };

    use crate::types::Kasten;

    #[tokio::test]
    async fn test_instantiation() {
        let path = PathBuf::from("/tmp/filaments/.filaments/filaments.db");
        create_dir_all(path.parent().unwrap()).unwrap();
        let _ = File::create(&path).unwrap();
        let _k = Kasten::index(dbg!(&path.parent().unwrap().parent().unwrap()))
            .await
            .unwrap();
    }
}
