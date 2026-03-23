//! Testing task functionality with the database abstraction.

use db::entity::{group, prelude::*, zettel};
use db::{ActiveValue::Set, entity::task};
use sea_orm::ActiveModelTrait;
mod common;

#[tokio::test]
async fn test_group_task_insert() {
    let db = common::fresh_test_db().await;

    let group_zettel: zettel::Model = zettel::ActiveModel {
        title: Set("Something".to_owned()),
        file_path: Set("/voo/doo".to_owned()),
        ..Default::default()
    }
    .insert(db.as_ref())
    .await
    .unwrap();

    let group: group::Model = group::ActiveModel {
        name: Set("something".to_owned()),
        color: Set("color".to_owned()),
        zettel_id: Set(group_zettel.nano_id.clone()),
        ..Default::default()
    }
    .insert(db.as_ref())
    .await
    .unwrap();

    let task_zettel: zettel::Model = zettel::ActiveModel {
        // nano_id: Set(NanoId::default()),
        title: Set("nomething".to_owned()),
        file_path: Set("/voo/doo".to_owned()),
        ..Default::default()
    }
    .insert(db.as_ref())
    .await
    .unwrap();

    let task: task::Model = task::ActiveModel {
        name: Set("something".to_owned()),
        group_id: Set(group.nano_id.to_owned()),
        zettel_id: Set(task_zettel.nano_id.clone()),
        ..Default::default()
    }
    .insert(db.as_ref())
    .await
    .unwrap();

    let task = Task::load()
        .filter_by_nano_id(task.nano_id.clone())
        .with(Group)
        .with(Zettel)
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();

    let group = Group::load()
        .filter_by_nano_id(group.nano_id.clone())
        .with(Task)
        .with(Zettel)
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();

    println!("group: {group:#?}");
    println!("task: {task:#?}");

    panic!()
}
