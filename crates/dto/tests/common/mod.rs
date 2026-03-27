use std::{
    fs::{File, create_dir_all},
    path::PathBuf,
};

use dto::Db;
use rand::RngExt;

pub async fn fresh_test_db() -> Db {
    let rand_id = {
        let mut rng = rand::rng();
        let mut rand_id = [0_u8; 4];
        rand_id.fill_with(|| rng.sample(rand::distr::Alphanumeric));

        String::from_utf8(rand_id.to_vec()).unwrap()
    };

    let path = PathBuf::from(format!("/tmp/filaments/test_db_{rand_id}"));

    create_dir_all(path.parent().unwrap()).unwrap();

    let _ = File::create(&path).unwrap();
    Db::connect(&path).await.unwrap()
}
