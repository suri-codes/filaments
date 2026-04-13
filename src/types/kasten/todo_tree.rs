use std::collections::HashMap;

use color_eyre::eyre::{Context, Result};
use dto::{
    ColumnTrait as _, DatabaseConnection, GroupColumns, GroupEntity, NanoId, QueryFilter as _,
    TagEntity, TaskColumns, TaskEntity, ZettelEntity,
};
use tree::{InsertBehavior, Node, NodeId, Tree};

use crate::types::{Group, Task};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TodoNodeKind {
    Root,
    Group(Box<Group>),
    Task(Box<Task>),
}

#[derive(Debug, Clone)]
pub struct TodoNode {
    pub depth: usize,
    pub kind: TodoNodeKind,
}

impl TodoNode {
    pub const fn new(kind: TodoNodeKind, depth: usize) -> Self {
        Self { depth, kind }
    }
}

#[derive(Debug)]
pub struct TodoTree {
    pub tree: Tree<TodoNode>,
    pub nanoid_to_nodeid: HashMap<NanoId, NodeId>,
    pub root_id: NodeId,
}

impl TodoTree {
    pub async fn construct(db: &DatabaseConnection) -> Result<Self> {
        let mut tree = Tree::<TodoNode>::new();
        let root_id = tree
            .insert(
                Node::new(TodoNode::new(TodoNodeKind::Root, 0)),
                InsertBehavior::AsRoot,
            )
            .with_context(|| "Could not create root node.")?;

        let root_groups: Vec<Group> = GroupEntity::load()
            .with(TagEntity)
            .with(TaskEntity)
            .with((ZettelEntity, TagEntity))
            .filter(GroupColumns::ParentGroupId.is_null())
            .all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        let mut todo_tree = Self {
            tree,
            nanoid_to_nodeid: HashMap::new(),
            root_id: root_id.clone(),
        };

        for group in root_groups {
            todo_tree
                .add_group_to_tree(db, &root_id, Box::new(group), 0)
                .await?;
        }

        Ok(todo_tree)
    }

    

    #[async_recursion::async_recursion]
    async fn add_group_to_tree(
        &mut self,
        db: &DatabaseConnection,
        parent_node_id: &NodeId,
        group: Box<Group>,
        depth: usize,
    ) -> Result<()> {
        let group_id = group.id.clone();

        let group_node_id = self.tree.insert(
            Node::new(TodoNode::new(TodoNodeKind::Group(group), depth)),
            InsertBehavior::UnderNode(parent_node_id),
        )?;

        self.nanoid_to_nodeid
            .insert(group_id.clone(), group_node_id.clone());

        let group_model = GroupEntity::load()
            .with(TagEntity)
            .with((ZettelEntity, TagEntity))
            .filter_by_nano_id(group_id.clone())
            .one(db)
            .await?
            .expect("We just inserted it");

        let tasks: Vec<Task> = TaskEntity::load()
            .with((ZettelEntity, TagEntity))
            .filter(TaskColumns::GroupId.eq(group_id.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|mut am| {
                am.group = dto::HasOne::Loaded(Box::new(group_model.clone()));
                am.into()
            })
            .collect();

        for task in tasks {
            let task_id = task.id.clone();
            let task_node_id = self.tree.insert(
                Node::new(TodoNode::new(TodoNodeKind::Task(Box::new(task)), depth + 1)),
                InsertBehavior::UnderNode(&group_node_id),
            )?;

            self.nanoid_to_nodeid.insert(task_id, task_node_id);
        }

        let children_groups: Vec<Group> = GroupEntity::load()
            .with(TagEntity)
            .with(TaskEntity)
            .with((ZettelEntity, TagEntity))
            .filter(GroupColumns::ParentGroupId.eq(group_id))
            .all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        for group in children_groups {
            self.add_group_to_tree(db, &group_node_id, Box::new(group), depth + 1)
                .await?;
        }

        Ok(())
    }
}
