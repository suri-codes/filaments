use dto::{
    ActiveModelTrait, ActiveValue::Set, ColorDTO, TagActiveModel, TagEntity, TagModel,
    ZettelActiveModel, ZettelEntity, ZettelModel,
};
use sea_orm::IntoActiveModel;

mod common;

#[tokio::test]
async fn test_zettel_tag_insert() {
    let db = common::fresh_test_db().await;

    let tag: TagModel = TagActiveModel {
        name: Set("Penis".to_owned()),
        color: Set(ColorDTO::new(255, 0, 0)),
        ..Default::default()
    }
    .insert(&*db)
    .await
    .unwrap();

    let _: ZettelModel = ZettelActiveModel {
        // nano_id: Set(NanoId::default()),
        title: Set("something1".to_owned()),
        file_path: Set("/voo/doo".to_owned()),
        ..Default::default()
    }
    .insert(db.as_ref())
    .await
    .unwrap();

    let x = ZettelActiveModel::builder()
        .set_title("Hello")
        .set_file_path("/voo/doo")
        // .add_tag(
        //     TagActiveModel::builder()
        //         .set_name("Hi")
        //         .set_color("some color"),
        // )
        .add_tag(tag.clone().into_active_model())
        .insert(&*db)
        .await
        .unwrap();

    dbg!(x);
    let _: ZettelModel = ZettelActiveModel {
        // nano_id: Set(NanoId::default()),
        title: Set("nomething2".to_owned()),
        file_path: Set("/voo/doo".to_owned()),
        ..Default::default()
    }
    .insert(db.as_ref())
    .await
    .unwrap();

    let zettels_for_tag = TagEntity::load()
        .filter_by_nano_id(tag.nano_id.clone())
        .with(ZettelEntity)
        .all(db.as_ref())
        .await
        .unwrap();

    dbg!(zettels_for_tag);
}
