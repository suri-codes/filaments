use std::{cmp::Ordering, collections::HashMap};

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
    pub p_score: f64,
}

impl TodoNode {
    pub const fn new(kind: TodoNodeKind, depth: usize, pscore: f64) -> Self {
        Self {
            depth,
            kind,
            p_score: pscore,
        }
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
                Node::new(TodoNode::new(TodoNodeKind::Root, 0, 1.0)),
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
                .add_group_to_tree(db, &root_id, Box::new(group), 0, 1.0)
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
        parent_p_score: f64,
    ) -> Result<()> {
        let group_id = group.id.clone();

        let p_score = group.p_score(parent_p_score);

        let group_node_id = self.tree.insert(
            Node::new(TodoNode::new(TodoNodeKind::Group(group), depth, p_score)),
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
            let p_score = task.p_score(p_score);

            let task_id = task.id.clone();
            let task_node_id = self.tree.insert(
                Node::new(TodoNode::new(
                    TodoNodeKind::Task(Box::new(task)),
                    depth + 1,
                    p_score,
                )),
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
            self.add_group_to_tree(db, &group_node_id, Box::new(group), depth + 1, p_score)
                .await?;
        }

        Ok(())
    }

    pub fn insert_group(&mut self, group: &Group) {
        let parent_node_id = group
            .parent_id
            .clone()
            .and_then(|id| self.nanoid_to_nodeid.get(&id))
            .unwrap_or(&self.root_id)
            .clone();

        let my_depth = if parent_node_id == self.root_id {
            0
        } else {
            self.tree
                .get(&parent_node_id)
                .expect("Must exist inside tree")
                .data()
                .depth
                + 1
        };

        let parent_p_score = self
            .tree
            .get(&parent_node_id)
            .expect("must exist")
            .data()
            .p_score;

        let my_pscore = parent_p_score * group.priority.p_score();

        let inserted_node_id = self
            .tree
            .insert(
                Node::new(TodoNode::new(
                    super::TodoNodeKind::Group(Box::new(group.clone())),
                    my_depth,
                    my_pscore,
                )),
                tree::InsertBehavior::UnderNode(&parent_node_id),
            )
            .expect("Insertion of group should not error!");

        self.reorder_chidren(&parent_node_id);

        self.nanoid_to_nodeid
            .insert(group.id.clone(), inserted_node_id);
    }

    pub fn insert_task(&mut self, task: &Task) {
        let parent_node_id = self
            .nanoid_to_nodeid
            .get(&task.group_id)
            .expect("The group must already be in the lookup hashmap")
            .clone();

        let my_depth = self
            .tree
            .get(&parent_node_id)
            .expect("Must exist inside tree")
            .data()
            .depth
            + 1;

        let parent_p_score = self
            .tree
            .get(&parent_node_id)
            .expect("must exist")
            .data()
            .p_score;

        let my_pscore = task.p_score(parent_p_score);

        let inserted_node_id = self
            .tree
            .insert(
                Node::new(TodoNode::new(
                    super::TodoNodeKind::Task(Box::new(task.clone())),
                    my_depth,
                    my_pscore,
                )),
                tree::InsertBehavior::UnderNode(&parent_node_id),
            )
            .expect("Insertion of Task should not error!");

        self.reorder_chidren(&parent_node_id);

        self.nanoid_to_nodeid
            .insert(task.id.clone(), inserted_node_id);
    }

    fn reorder_chidren(&mut self, parent_node_id: &NodeId) {
        let children = self
            .tree
            .children(parent_node_id)
            .expect("Must be valid")
            .zip(
                self.tree
                    .children_ids(parent_node_id)
                    .expect("Must be valid"),
            )
            .map(|(a, b)| (b.clone(), matches!(a.data().kind, TodoNodeKind::Task(_))))
            .collect::<HashMap<_, _>>();

        let parent = self
            .tree
            .get_mut(parent_node_id)
            .expect("parent must exist");

        parent.sort_children_by(|a, _| {
            let a = children.get(a).expect("must exist");

            if *a {
                return Ordering::Less;
            }

            Ordering::Equal
        });
    }

    pub fn get_node_by_nano_id(&self, nano_id: &NanoId) -> &Node<TodoNode> {
        let node_id = self
            .nanoid_to_nodeid
            .get(nano_id)
            .expect("invariant broken!");

        self.tree.get(node_id).expect("Invariant Broken!")
    }

    pub fn get_node_mut_by_nano_id(&mut self, nano_id: &NanoId) -> &mut Node<TodoNode> {
        let node_id = self
            .nanoid_to_nodeid
            .get(nano_id)
            .expect("invariant broken!");

        self.tree.get_mut(node_id).expect("Invariant Broken!")
    }
}
