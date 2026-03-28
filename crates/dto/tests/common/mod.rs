use std::{
    fs::{File, create_dir_all},
    path::PathBuf,
};

use dto::{Migrator, MigratorTrait};
use rand::RngExt;
use sea_orm::{Database, DatabaseConnection};

pub async fn fresh_test_db() -> DatabaseConnection {
    let rand_id = {
        let mut rng = rand::rng();
        let mut rand_id = [0_u8; 4];
        rand_id.fill_with(|| rng.sample(rand::distr::Alphanumeric));

        String::from_utf8(rand_id.to_vec()).unwrap()
    };

    let path = PathBuf::from(format!("/tmp/filaments/test_db_{rand_id}.db"));

    create_dir_all(path.parent().unwrap()).unwrap();

    let _ = File::create(&path).unwrap();

    let db_conn_string = format!(
        "sqlite://{}",
        path.clone().canonicalize().unwrap().to_string_lossy()
    );

    let conn = Database::connect(db_conn_string).await.unwrap();

    // run da migrations every time we connect, just in case
    Migrator::up(&conn, None).await.unwrap();

    conn
}
