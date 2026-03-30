//! Testing task functionality with the database abstraction.

use dto::{
    ActiveModelTrait as _, ActiveValue::Set, GroupActiveModel, GroupEntity, GroupModel,
    TaskActiveModel, TaskEntity, TaskModel, ZettelActiveModel, ZettelEntity, ZettelModel,
};
use migration::types::Color;
mod common;

#[tokio::test]
async fn test_group_task_insert() {
    let db = common::fresh_test_db().await;

    let group_zettel: ZettelModel = ZettelActiveModel {
        title: Set("Something".to_owned()),
        file_path: Set("/voo/doo".to_owned()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let group: GroupModel = GroupActiveModel {
        name: Set("something".to_owned()),
        color: Set(Color::new(255, 0, 0)),
        zettel_id: Set(group_zettel.nano_id.clone()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let task_zettel: ZettelModel = ZettelActiveModel {
        // nano_id: Set(NanoId::default()),
        title: Set("nomething".to_owned()),
        file_path: Set("/voo/doo".to_owned()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let task: TaskModel = TaskActiveModel {
        name: Set("something".to_owned()),
        group_id: Set(group.nano_id.to_owned()),
        zettel_id: Set(task_zettel.nano_id.clone()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let task = TaskEntity::load()
        .filter_by_nano_id(task.nano_id.clone())
        .with(GroupEntity)
        .with(ZettelEntity)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let group = GroupEntity::load()
        .filter_by_nano_id(group.nano_id.clone())
        .with(TaskEntity)
        .with(ZettelEntity)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    println!("group: {group:#?}");
    println!("task: {task:#?}");
    // panic!()
}
