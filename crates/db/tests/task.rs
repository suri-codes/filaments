//! Testing task functionality with the database abstraction.

use db::entity::{group, prelude::*};
use db::{ActiveValue::Set, entity::task};
use sea_orm::ActiveModelTrait;
mod common;

#[tokio::test]
async fn test_group_task_insert() {
    let db = common::fresh_test_db().await;

    let group = group::ActiveModel {
        name: Set("something".to_owned()),
        color: Set("color".to_owned()),
        description_path: Set("something".to_owned()),
        ..Default::default()
    };

    let group: group::Model = group.insert(db.as_ref()).await.unwrap();

    let task = task::ActiveModel {
        name: Set("something".to_owned()),
        description_path: Set("something".to_owned()),
        group_id: Set(group.nano_id.to_owned()),
        ..Default::default()
    };

    let task: task::Model = task.insert(db.as_ref()).await.unwrap();

    let task = Task::find_by_nano_id(task.nano_id)
        .inner_join(Group)
        // .reverse_join(Group)
        // .find_with_related(Group)
        .all(db.as_ref())
        .await
        .unwrap();

    let task = Task::load()
        .filter_by_nano_id(task.first().unwrap().nano_id.clone())
        .with(Group)
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();

    println!("{group:#?}");
    println!("{task:#?}");
}
