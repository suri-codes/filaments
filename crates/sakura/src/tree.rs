use std::cmp::Ordering;

use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::{
    Ancestors, Children, ChildrenIds, InsertBehavior, MoveBehavior, Node, NodeId,
    PreOrderTraversal, PreOrderTraversalIds, RemoveBehavior, error::NodeIdError,
    iterators::AncestorsIds,
};

/// A `Tree` builder to assist with building a `Tree`, with more control.
pub struct TreeBuilder<T> {
    root: Option<Node<T>>,
    node_capacity: usize,
    swap_capacity: usize,
}

impl<T> Default for TreeBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TreeBuilder<T> {
    /// Creates a new `TreeBuilder` with default settings.
    ///
    /// ```
    /// use sakura::TreeBuilder;
    ///
    /// let _tree_builder: TreeBuilder<i32> = TreeBuilder::new();
    ///
    /// ```
    #[allow(clippy::use_self)]
    #[must_use]
    pub const fn new() -> TreeBuilder<T> {
        TreeBuilder {
            root: None,
            node_capacity: 0,
            swap_capacity: 0,
        }
    }

    /// Sets the root `Node` for the resulting `Tree` from this `TreeBuilder`.
    ///
    /// ```
    /// use sakura::TreeBuilder;
    /// use sakura::Node;
    ///
    /// let _tree_builder: TreeBuilder<i32> = TreeBuilder::new().with_root(Node::new(1));
    /// ```
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn with_root(mut self, root: Node<T>) -> TreeBuilder<T> {
        self.root = Some(root);
        self
    }

    /// Sets the `node_capacity` for `TreeBuilder`.
    ///
    /// Since `Tree`'s own their `Node`'s, they must allocate
    /// storage for `Node`'s ahead of time, so that the
    /// space allocations don't happen as the `Node`'s are inserted.
    ///
    /// _Configure this variable if you know the **maximum** number of `Node`'s
    /// that your `Tree` will **contain** at **any given time**._
    ///
    /// ```
    /// use sakura::TreeBuilder;
    ///
    /// let _tree_builder: TreeBuilder<i32> = TreeBuilder::new().with_node_capacity(1);
    ///
    /// ```
    #[must_use]
    #[allow(clippy::use_self)]
    pub const fn with_node_capacity(mut self, node_capacity: usize) -> TreeBuilder<T> {
        self.node_capacity = node_capacity;
        self
    }

    /// Sets the `swap_capacity` for `TreeBuilder`.
    ///
    /// `Tree`'s attempt to save time by reusing storage space
    /// when `Node`'s are removed (instead of shuffling `Node`'s around internally).
    /// To do this, the `Tree` must store information about the space left behind when a `Node`
    /// is removed. Using this setting allows the `Tree` to pre-allocate this storage
    /// space instead of doing so as `Node`'s are removed from the `Tree`.
    ///
    /// _Use of this setting is recommended if you know the **maximum "net number
    /// of removals"** that have occurred at **any given time**._
    ///
    ///
    /// For example:
    /// ---
    /// In **Scenario 1**:
    ///
    /// * Add 3 `Node`s, Remove 2 `Node`s, Add 1 `Node`.
    ///
    /// The maximum amount of nodes that have been removed at any given time is **2**.
    ///
    /// But in **Scenario 2**:
    ///
    /// * Add 3 `Node`s, Remove 2 `Node`s, Add 1 `Node`, Remove 2 `Node`s.
    ///
    /// The maximum amount of nodes that have been removed at any given time is **3**.
    ///
    /// ```
    /// use sakura::TreeBuilder;
    ///
    /// let _tree_builder: TreeBuilder<i32> = TreeBuilder::new().with_node_capacity(1);
    ///
    /// ```
    #[must_use]
    #[allow(clippy::use_self)]
    pub const fn with_swap_capacity(mut self, swap_capacity: usize) -> TreeBuilder<T> {
        self.swap_capacity = swap_capacity;
        self
    }

    ///
    /// Build a `Tree` based upon the current settings in the `TreeBuilder`.
    ///
    /// ```
    /// use sakura::TreeBuilder;
    /// use sakura::Tree;
    /// use sakura::Node;
    ///
    /// let _tree: Tree<i32> = TreeBuilder::new()
    ///         .with_root(Node::new(5))
    ///         .with_node_capacity(3)
    ///         .with_swap_capacity(2)
    ///         .build();
    /// ```
    pub fn build(mut self) -> Tree<T> {
        let mut tree = Tree {
            root: None,
            nodes: Vec::with_capacity(self.node_capacity),
            free_ids: Vec::with_capacity(self.swap_capacity),
        };

        if self.root.is_some() {
            let node_id = NodeId { index: 0 };

            tree.nodes.push(self.root.take());

            tree.root = Some(node_id);
        }

        tree
    }
}

/// A tree structure made up of `Node`'s.
///
/// # Panics
/// Any function that takes a `NodeId` can `panic`, but this should
/// only happen with improper `NodeId` management within `Sakura`, and
/// should have nothing to do with library user's code.
#[derive(Debug, Serialize, Deserialize, Reconcile, Hydrate)]
pub struct Tree<T> {
    root: Option<NodeId>,
    pub(crate) nodes: Vec<Option<Node<T>>>,
    free_ids: Vec<NodeId>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PartialEq for Tree<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.nodes.iter().filter(|x| x.is_some()).count()
            != other.nodes.iter().filter(|x| x.is_some()).count()
        {
            return false;
        }

        for ((i, node1), (j, node2)) in self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, x)| (*x).as_ref().map(|x| (i, x)))
            .zip(
                other
                    .nodes
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| (*x).as_ref().map(|x| (i, x))),
            )
        {
            let parent1_node = node1.parent.as_ref().and_then(|x| self.get(x).ok());
            let parent2_node = node2.parent.as_ref().and_then(|x| other.get(x).ok());

            if i != j || node1 != node2 || parent1_node != parent2_node {
                return false;
            }
        }

        true
    }
}

impl<T> Tree<T> {
    /// Creates a new `Tree` with default settings (no root `Node` and no space pre-allocation)
    ///
    /// ```
    /// use sakura::Tree;
    ///
    /// let _tree: Tree<i32> = Tree::new();
    /// ```
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new() -> Tree<T> {
        TreeBuilder::new().build()
    }

    ///
    /// Returns the number of elements the tree can hold without reallocating.
    ///
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Returns a `Some` value containing the `NodeId` of the root `Node` if
    /// it exists. Otherwise, a `None` is returned.
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(5), AsRoot).unwrap();
    ///
    /// # assert_eq!(&root_id, tree.root_node_id().unwrap());
    /// ```
    ///
    #[must_use]
    pub const fn root_node_id(&self) -> Option<&NodeId> {
        self.root.as_ref()
    }

    /// Returns the maximum height of the `Tree`.
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// # assert_eq!(0, tree.height());
    ///
    /// let root_id = tree.insert(Node::new(1), AsRoot).unwrap();
    /// # assert_eq!(1, tree.height());
    ///
    /// tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
    /// # assert_eq!(2, tree.height());
    /// ```
    #[must_use]
    pub fn height(&self) -> usize {
        self.root
            .as_ref()
            .map_or_else(|| 0, |id| self.height_of_node(id))
    }

    fn height_of_node(&self, node: &NodeId) -> usize {
        let mut h = 0;
        for n in self.children_ids(node).unwrap() {
            h = std::cmp::max(h, self.height_of_node(n));
        }

        h + 1
    }

    /// Gets a reference `Node` from the `Tree`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(5), AsRoot).unwrap();
    ///
    /// let root_node: &Node<i32> = tree.get(&root_id).unwrap();
    ///
    /// # assert_eq!(root_node.data(), &5);
    /// ```
    ///
    pub fn get(&self, node_id: &NodeId) -> Result<&Node<T>, NodeIdError> {
        // Returns if node id isn't valid.
        let () = self.is_valid_node_id(node_id)?;

        self.nodes
            .get(node_id.index as usize)
            .expect(
                "index must
            exist in tree",
            )
            .as_ref()
            // Since we are given a node id, and that entry in the nodes
            // vec isn't an actual node, the node_id is no longer valid.
            .ok_or(NodeIdError::NodeIdNoLongerValid)
    }

    /// Gets a mutable reference to a `Node` from the `Tree`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(5), AsRoot).unwrap();
    ///
    /// let root_node: &mut Node<i32> = tree.get_mut(&root_id).unwrap();
    ///
    /// # assert_eq!(root_node.data(), &5);
    /// ```
    ///
    pub fn get_mut(&mut self, node_id: &NodeId) -> Result<&mut Node<T>, NodeIdError> {
        // Returns if node id isn't valid.
        let () = self.is_valid_node_id(node_id)?;

        self.nodes
            .get_mut(node_id.index as usize)
            .expect(
                "index must
            exist in tree",
            )
            .as_mut()
            // Since we are given a node id, and that entry in the nodes
            // vec isn't an actual node, the node_id is no longer valid.
            .ok_or(NodeIdError::NodeIdNoLongerValid)
    }

    /// Inserts a `Node` into the `Tree`, via the provided `InsertBehavior`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let root_node = Node::new(1);
    /// let child_node = Node::new(2);
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(root_node, AsRoot).unwrap();
    ///
    /// tree.insert(child_node, UnderNode(&root_id)).unwrap();
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn insert(
        &mut self,
        node: Node<T>,
        behavior: InsertBehavior,
    ) -> Result<NodeId, NodeIdError> {
        match behavior {
            InsertBehavior::AsRoot => Ok(self.set_root(node)),
            InsertBehavior::UnderNode(parent_id) => {
                self.is_valid_node_id(parent_id)?;
                Ok(self.insert_with_parent(node, parent_id))
            }
        }
    }

    /// Removes a `Node` from the `Tree`, via the provided `RemoveBehavior`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    /// use sakura::RemoveBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    ///
    /// let child_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// let grandchild_id = tree.insert(Node::new(2), UnderNode(&child_id)).unwrap();
    ///
    /// let child = tree.remove_node(child_id, DropChildren).unwrap();
    ///
    /// # assert!(tree.get(&grandchild_id).is_err());
    /// # assert_eq!(tree.get(&root_id).unwrap().children().len(), 0);
    /// # assert_eq!(child.children().len(), 0);
    /// # assert_eq!(child.parent(), None);
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_node(
        &mut self,
        node_id: NodeId,
        behavior: RemoveBehavior,
    ) -> Result<Node<T>, NodeIdError> {
        self.is_valid_node_id(&node_id)?;

        match behavior {
            RemoveBehavior::DropChildren => Ok(self.remove_node_drop_children(node_id)),
            RemoveBehavior::LiftChildren => Ok(self.remove_node_lift_children(node_id)),
            RemoveBehavior::OrphanChildren => Ok(self.remove_node_orphan_children(node_id)),
        }
    }

    /// Remove a `Node` from the `Tree`, while transferring all of its children
    /// to its parent.
    fn remove_node_lift_children(&mut self, node_id: NodeId) -> Node<T> {
        if let Some(parent_id) = self
            .get(&node_id)
            .expect("Tree::remove_node_lift_children: Expecting node_id to be valid.")
            .parent()
            .cloned()
        {
            for child_id in self
                .get(&node_id)
                .expect("Tree::remove_node_lift_children: Expecting node_id to be valid.")
                .children()
                .clone()
            {
                self.set_as_parent_and_child(&parent_id, &child_id);
            }
        } else {
            self.clear_parent_of_children(&node_id);
        }

        self.remove_node_internal(node_id)
    }

    /// Remove a `Node` from the `Tree` including all of its children recursively.
    fn remove_node_drop_children(&mut self, node_id: NodeId) -> Node<T> {
        let children = self
            .get_mut(&node_id)
            .expect("Tree::remove_node_drop_children: Expecting node_id to be valid.")
            .take_children();

        for child in children {
            self.remove_node_drop_children(child);
        }
        self.remove_node_internal(node_id)
    }

    /// Remove a `node` from the `Tree` and leave all of its children in the `Tree`
    fn remove_node_orphan_children(&mut self, node_id: NodeId) -> Node<T> {
        self.clear_parent_of_children(&node_id);
        self.remove_node_internal(node_id)
    }

    /// Moves a `Node` in the `Tree`, via the provided `MoveBehavior`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    #[allow(clippy::needless_pass_by_value)]
    pub fn move_node(
        &mut self,
        node_id: &NodeId,
        behavior: MoveBehavior,
    ) -> Result<(), NodeIdError> {
        self.is_valid_node_id(node_id)?;

        match behavior {
            MoveBehavior::ToRoot => {
                self.move_node_to_root(node_id);
                Ok(())
            }
            MoveBehavior::ToParent(parent_id) => {
                self.move_node_to_parent(node_id, parent_id);
                Ok(())
            }
        }
    }

    fn move_node_to_parent(&mut self, node_id: &NodeId, parent_id: &NodeId) {
        if let Some(subtree_root_id) = self
            .find_subtree_root_between_ids(parent_id, node_id)
            .cloned()
        {
            // node_id is above parent_id, this is a move "down" the tree
            let root = self.root.clone();

            if root.as_ref() == Some(node_id) {
                // We're moving the root down the tree.
                // Also we know the root exists.

                // Detach subtree_root from node.
                self.detach_from_parent(node_id, &subtree_root_id);

                // Set subtree_root as Tree root.
                self.clear_parent(&subtree_root_id);
                self.root = Some(subtree_root_id);
            } else {
                // We're moving some other node down the tree.

                if let Some(old_parent) = self
                    .get(node_id)
                    .expect("Tree::move_node_to_parent: Expecting valid node_id")
                    .parent()
                    .cloned()
                {
                    // Detach from old parent.
                    self.detach_from_parent(&old_parent, node_id);

                    //Connect old parent and subtree root.
                    self.set_as_parent_and_child(&old_parent, &subtree_root_id);
                } else {
                    // Node is orphaned, need to set subtree_root's parent to None (same as node's).

                    self.clear_parent(&subtree_root_id);
                }

                // Detach subtree_root from node.
                self.detach_from_parent(node_id, &subtree_root_id);
            }
        } else {
            // this is a move "across" or "up" the tree

            // detach from old parent

            if let Some(old_parent) = self
                .get(node_id)
                .expect("Tree::move_node_to_parent: Expecting valid node_id")
                .parent()
                .cloned()
            {
                self.detach_from_parent(&old_parent, node_id);
            }
        }
        self.set_as_parent_and_child(parent_id, node_id);
    }

    /// Sorts the children of a `Node`, in-place, using compare to compare
    /// the nodes
    ///
    /// This sort is stable and O(n log n) worst case but allocates
    /// approximately 2 * n where n is the length of children
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(100), AsRoot).unwrap();
    /// tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
    /// tree.insert(Node::new(0), UnderNode(&root_id)).unwrap();
    ///
    /// tree.sort_children_by(&root_id, |a, b| a.data().cmp(b.data())).unwrap();
    ///
    /// # for (i, id) in tree.get(&root_id).unwrap().children().iter().enumerate() {
    /// #   assert_eq!(*tree.get(&id).unwrap().data(), i as i32);
    /// # }
    /// ```
    pub fn sort_children_by<F>(
        &mut self,
        node_id: &NodeId,
        mut compare: F,
    ) -> Result<(), NodeIdError>
    where
        F: FnMut(&Node<T>, &Node<T>) -> Ordering,
    {
        self.is_valid_node_id(node_id)?;

        let mut children = self
            .get_mut(node_id)
            .expect("Tree::sort_children_by: expecting to be passed in a valid node_id")
            .take_children();

        children.sort_by(|a, b| {
            compare(
                self.get(a)
                    .expect("Tree::sort_children_by: expecting to be passed in a valid node_id"),
                self.get(b)
                    .expect("Tree::sort_children_by: expecting to be passed in a valid node_id"),
            )
        });

        self.get_mut(node_id)
            .expect("Tree::sort_children_by: expecting to be passed in a valid node_id")
            .set_children(children);

        Ok(())
    }

    /// Sorts the children of a `Node`, in-place, using their data.
    ///
    /// This sort is stable and O(n log n) worst case but allocates
    /// approximately 2 * n where n is the length of children
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(100), AsRoot).unwrap();
    /// tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
    /// tree.insert(Node::new(0), UnderNode(&root_id)).unwrap();
    ///
    /// tree.sort_children_by_data(&root_id).unwrap();
    ///
    /// # for (i, id) in tree.get(&root_id).unwrap().children().iter().enumerate() {
    /// #   assert_eq!(*tree.get(&id).unwrap().data(), i as i32);
    /// # }
    /// ```
    ///
    pub fn sort_children_by_data(&mut self, node_id: &NodeId) -> Result<(), NodeIdError>
    where
        T: Ord,
    {
        self.is_valid_node_id(node_id)?;

        let mut children = self
            .get_mut(node_id)
            .expect("Tree::sort_children_by: expecting to be passed in a valid node_id")
            .take_children();

        children.sort_by_key(|a| {
            self.get(a)
                .expect("Tree::sort_children_by: expecting to be passed in a valid node_id")
        });

        self.get_mut(node_id)
            .expect("Tree::sort_children_by: expecting to be passed in a valid node_id")
            .set_children(children);

        Ok(())
    }

    /// Returns an `Ancestors` iterator
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let node_1 = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    ///
    /// let mut ancestors = tree.ancestors(&node_1).unwrap();
    ///
    /// # assert_eq!(ancestors.next().unwrap().data(), &0);
    /// # assert!(ancestors.next().is_none());
    /// ```
    pub fn ancestors(&self, node_id: &NodeId) -> Result<Ancestors<'_, T>, NodeIdError> {
        self.is_valid_node_id(node_id)?;
        Ok(Ancestors::new(self, node_id.clone()))
    }

    /// Returns an `AncestorIds` iterator
    ///
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let node_1 = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    ///
    /// let mut ancestor_ids = tree.ancestor_ids(&node_1).unwrap();
    ///
    /// # assert_eq!(ancestor_ids.next().unwrap(), &root_id);
    /// # assert!(ancestor_ids.next().is_none());
    /// ```
    ///
    pub fn ancestor_ids(&self, node_id: &NodeId) -> Result<AncestorsIds<'_, T>, NodeIdError> {
        self.is_valid_node_id(node_id)?;

        Ok(AncestorsIds::new(self, node_id.clone()))
    }

    /// Returns an `Children` iterator for a given `NodeId`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    ///
    /// let mut children = tree.children(&root_id).unwrap();
    ///
    /// # assert_eq!(children.next().unwrap().data(), &1);
    /// # assert!(children.next().is_none());
    /// ```
    pub fn children(&self, node_id: &NodeId) -> Result<Children<'_, T>, NodeIdError> {
        self.is_valid_node_id(node_id)?;
        Ok(Children::new(self, node_id))
    }

    /// Returns an `Children` iterator for a given `NodeId`
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let node_1 = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    ///
    /// let mut children_ids = tree.children_ids(&root_id).unwrap();
    ///
    /// # assert_eq!(children_ids.next().unwrap(), &node_1);
    /// # assert!(children_ids.next().is_none());
    /// ```
    pub fn children_ids(&self, node_id: &NodeId) -> Result<ChildrenIds<'_>, NodeIdError> {
        self.is_valid_node_id(node_id)?;
        Ok(ChildrenIds::new(self, node_id))
    }

    /// Returns a `PreOrderTraversal` iterator
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    ///
    /// let mut nodes = tree.traverse_pre_order(&root_id).unwrap();
    ///
    /// # assert_eq!(nodes.next().unwrap().data(), &0);
    /// # assert_eq!(nodes.next().unwrap().data(), &1);
    /// # assert!(nodes.next().is_none());
    /// ```
    ///
    pub fn traverse_pre_order(
        &self,
        node_id: &NodeId,
    ) -> Result<PreOrderTraversal<'_, T>, NodeIdError> {
        self.is_valid_node_id(node_id)?;

        Ok(PreOrderTraversal::new(self, node_id.clone()))
    }

    /// Returns a `PreOrderTraversalIds` iterator
    ///
    /// # Errors
    ///
    /// Can error if the given `NodeId` is not valid (i.e. it was removed from the `Tree`.)
    ///
    /// # Panics
    ///
    /// Can panic if the `NodeId` does not exist in the `Tree`, but this would
    /// be a bug in `Sakura`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    ///
    /// let mut nodes = tree.traverse_pre_order_ids(&root_id).unwrap();
    ///
    /// assert_eq!(tree.get(&nodes.next().unwrap()).unwrap().data(), &0);
    /// assert_eq!(tree.get(&nodes.next().unwrap()).unwrap().data(), &1);
    /// assert!(nodes.next().is_none());
    /// ```
    ///
    pub fn traverse_pre_order_ids(
        &self,
        node_id: &NodeId,
    ) -> Result<PreOrderTraversalIds<'_, T>, NodeIdError> {
        self.is_valid_node_id(node_id)?;

        Ok(PreOrderTraversalIds::new(self, node_id.clone()))
    }

    fn move_node_to_root(&mut self, node_id: &NodeId) {
        let old_root = self.root.clone();

        if let Some(parent_id) = self
            .get(node_id)
            .expect("Tree::move_node_to_root Expected a valid NodeId")
            .parent
            .clone()
        {
            self.detach_from_parent(&parent_id, node_id);
        }

        self.clear_parent(node_id);

        self.root = Some(node_id.clone());

        if let Some(old_root) = old_root {
            self.move_node_to_parent(&old_root, node_id);
        }
    }

    fn insert_with_parent(&mut self, child: Node<T>, parent_id: &NodeId) -> NodeId {
        let new_child_id = self.insert_new_node(child);
        self.set_as_parent_and_child(parent_id, &new_child_id);
        new_child_id
    }

    fn set_root(&mut self, new_root: Node<T>) -> NodeId {
        let new_root_id = self.insert_new_node(new_root);

        if let Some(current_root_node_id) = self.root.clone() {
            self.set_as_parent_and_child(&new_root_id, &current_root_node_id);
        }

        self.root = Some(new_root_id.clone());

        new_root_id
    }

    fn find_subtree_root_between_ids<'a>(
        &'a self,
        lower_id: &'a NodeId,
        upper_id: &'a NodeId,
    ) -> Option<&'a NodeId> {
        if let Some(lower_parent) = self.get(lower_id).unwrap().parent() {
            if lower_parent == upper_id {
                return Some(lower_id);
            }
            return self.find_subtree_root_between_ids(lower_parent, upper_id);
        }

        None
    }

    fn set_as_parent_and_child(&mut self, parent_id: &NodeId, child_id: &NodeId) {
        self.get_mut(parent_id)
            .expect("Tree::set_as_parent_and_child: parent_id should be inside the Tree.")
            .add_child(child_id.clone());

        self.get_mut(child_id)
            .expect("Tree::set_as_parent_and_child: child_id should be inside the Tree.")
            .set_parent(Some(parent_id.clone()));
    }

    fn detach_from_parent(&mut self, parent_id: &NodeId, node_id: &NodeId) {
        self.get_mut(parent_id)
            .expect("Tree::detach_from_parent: parent_id must be present in tree")
            .children_mut()
            .retain(|child_id| *child_id != *node_id);
    }

    fn insert_new_node(&mut self, new_node: Node<T>) -> NodeId {
        if self.free_ids.is_empty() {
            let new_node_idx = self.nodes.len();
            self.nodes.push(Some(new_node));
            NodeId::new(new_node_idx)
        } else {
            let new_node_id = self
                .free_ids
                .pop()
                .expect("Tree::insert_new_node: Couldn't pop from vec with len() > 0.");

            self.nodes.push(Some(new_node));
            self.nodes.swap_remove(new_node_id.index as usize);
            new_node_id
        }
    }

    fn is_valid_node_id(&self, node_id: &NodeId) -> Result<(), NodeIdError> {
        let idx = node_id.index as usize;

        assert!(
            idx <= self.nodes.len(),
            "NodeId: {node_id:?} is out of bounds. This is a bug inside
            Sakura.",
        );

        if self.nodes.get(idx).is_none() {
            return Err(NodeIdError::NodeIdNoLongerValid);
        }

        Ok(())
    }

    // We want to have the node_id be consumed by this remove function.
    #[allow(clippy::needless_pass_by_value)]
    fn remove_node_internal(&mut self, node_id: NodeId) -> Node<T> {
        if let Some(root_id) = &self.root
            && node_id == *root_id
        {
            self.root = None;
        }

        let mut node = self.take_node(node_id.clone());

        if let Some(parent_id) = node.parent() {
            self.get_mut(parent_id)
                .expect(
                    "Tree::remove_node_internal: expecting
                parent_id to be a valid node_id!",
                )
                .children_mut()
                .retain(|child_id| *child_id != node_id);
        }

        node.children_mut().clear();
        node.set_parent(None);

        node
    }

    fn take_node(&mut self, node_id: NodeId) -> Node<T> {
        self.nodes.push(None);

        let node = self
            .nodes
            .swap_remove(node_id.index as usize)
            .expect("Tree::take_node: expecting node_id to be a valid node_id!");

        self.free_ids.push(node_id);

        node
    }

    fn clear_parent(&mut self, node_id: &NodeId) {
        self.set_parent(node_id, None);
    }

    fn set_parent(&mut self, node_id: &NodeId, parent_id: Option<NodeId>) {
        self.get_mut(node_id)
            .expect(
                "Tree::set_parent: expecting node_id to
            be present inside tree!",
            )
            .set_parent(parent_id);
    }

    fn clear_parent_of_children(&mut self, node_id: &NodeId) {
        self.set_parent_of_children(node_id, None);
    }

    fn set_parent_of_children(&mut self, node_id: &NodeId, new_parent: Option<&NodeId>) {
        for child_id in self
            .get(node_id)
            .expect("Tree::set_parent_of_child: expect node_id to be a valid node inside tree.")
            .children
            .clone()
        {
            self.set_parent(&child_id, new_parent.cloned());
        }
    }
}

impl<T: std::fmt::Debug> Tree<T> {
    /// Write formatted tree representation and nodes with debug formatting.
    ///
    ///
    /// # Errors
    ///
    /// Function can error if something goes wrong during debug!
    ///
    /// # Panics
    ///
    /// Function can error if something goes wrong during debug!
    ///
    /// ```
    /// use sakura::Tree;
    /// use sakura::Node;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree = Tree::<i32>::new();
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let first_child_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// let _ = tree.insert(Node::new(2), UnderNode(&first_child_id)).unwrap();
    /// let _ = tree.insert(Node::new(3), UnderNode(&root_id)).unwrap();
    /// let mut s = String::new();
    /// tree.write_formatted(&mut s).unwrap();
    /// assert_eq!(&s, "\
    /// 0
    /// ├── 1
    /// │   └── 2
    /// └── 3
    /// ");
    /// ```
    ///
    /// Writes nothing if the tree is empty.
    ///
    /// ```
    /// use sakura::Tree;
    ///
    /// let tree = Tree::<i32>::new();
    /// let mut s = String::new();
    /// tree.write_formatted(&mut s).unwrap();
    /// # assert_eq!(&s, "");
    /// ```
    pub fn write_formatted<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        if let Some(node_id) = self.root_node_id() {
            let childn = 0;
            let level = 0;
            let last = vec![];
            let mut stack = vec![(node_id, childn, level, last)];
            while let Some((node_id, childn, level, last)) = stack.pop() {
                debug_assert_eq!(
                    last.len(),
                    level,
                    "each previous level should indicate whether it has reached the last node"
                );
                let node = self
                    .get(node_id)
                    .expect("getting node of existing node ref id");
                if childn == 0 {
                    for i in 1..level {
                        if last[i - 1] {
                            write!(w, "    ")?;
                        } else {
                            write!(w, "│   ")?;
                        }
                    }
                    if level > 0 {
                        if last[level - 1] {
                            write!(w, "└── ")?;
                        } else {
                            write!(w, "├── ")?;
                        }
                    }
                    writeln!(w, "{:?}", node.data())?;
                }
                let mut children = node.children().iter().skip(childn);
                if let Some(child_id) = children.next() {
                    let mut next_last = last.clone();
                    if children.next().is_some() {
                        stack.push((node_id, childn + 1, level, last));
                        next_last.push(false);
                    } else {
                        next_last.push(true);
                    }
                    stack.push((child_id, 0, level + 1, next_last));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tree_builder_tests {
    use super::super::Node;
    use super::TreeBuilder;

    #[test]
    fn test_new() {
        let tb: TreeBuilder<i32> = TreeBuilder::new();
        assert!(tb.root.is_none());
        assert_eq!(tb.node_capacity, 0);
        assert_eq!(tb.swap_capacity, 0);
    }

    #[test]
    fn test_with_root() {
        let tb: TreeBuilder<i32> = TreeBuilder::new().with_root(Node::new(5));

        assert_eq!(tb.root.unwrap().data(), &5);
        assert_eq!(tb.node_capacity, 0);
        assert_eq!(tb.swap_capacity, 0);
    }

    #[test]
    fn test_with_node_capacity() {
        let tb: TreeBuilder<i32> = TreeBuilder::new().with_node_capacity(10);

        assert!(tb.root.is_none());
        assert_eq!(tb.node_capacity, 10);
        assert_eq!(tb.swap_capacity, 0);
    }

    #[test]
    fn test_with_swap_capacity() {
        let tb: TreeBuilder<i32> = TreeBuilder::new().with_swap_capacity(10);

        assert!(tb.root.is_none());
        assert_eq!(tb.node_capacity, 0);
        assert_eq!(tb.swap_capacity, 10);
    }

    #[test]
    fn test_with_all_settings() {
        let tb: TreeBuilder<i32> = TreeBuilder::new()
            .with_root(Node::new(5))
            .with_node_capacity(10)
            .with_swap_capacity(3);

        assert_eq!(tb.root.unwrap().data(), &5);
        assert_eq!(tb.node_capacity, 10);
        assert_eq!(tb.swap_capacity, 3);
    }

    #[test]
    fn test_build() {
        let tree = TreeBuilder::new()
            .with_root(Node::new(5))
            .with_node_capacity(10)
            .with_swap_capacity(3)
            .build();

        let root = tree.get(tree.root_node_id().unwrap()).unwrap();

        assert_eq!(root.data(), &5);
        assert_eq!(tree.capacity(), 10);
        assert_eq!(tree.free_ids.capacity(), 3);
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tree_tests {
    use crate::InsertBehavior;
    use crate::MoveBehavior;
    use crate::RemoveBehavior;

    use super::super::Node;
    use super::super::NodeId;
    use super::Tree;
    use super::TreeBuilder;

    #[test]
    fn test_new() {
        let tree: Tree<i32> = Tree::new();

        assert_eq!(tree.root, None);
        assert_eq!(tree.nodes.len(), 0);
        assert_eq!(tree.free_ids.len(), 0);
    }

    #[test]
    fn test_get() {
        let tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();
        let root = tree.get(&root_id).unwrap();

        assert_eq!(root.data(), &5);
    }

    #[test]
    fn test_get_mut() {
        let mut tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();

        {
            let root = tree.get(&root_id).unwrap();
            assert_eq!(root.data(), &5);
        }

        {
            let root = tree.get_mut(&root_id).unwrap();
            *root.data_mut() = 6;
        }

        let root = tree.get(&root_id).unwrap();
        assert_eq!(root.data(), &6);
    }

    #[test]
    fn test_set_root() {
        use InsertBehavior::*;

        let a = 5;
        let b = 6;
        let node_a = Node::new(a);
        let node_b = Node::new(b);

        let mut tree = TreeBuilder::new().build();

        let node_a_id = tree.insert(node_a, AsRoot).unwrap();
        let root_id = tree.root.clone().unwrap();
        assert_eq!(node_a_id, root_id);

        {
            let node_a_ref = tree.get(&node_a_id).unwrap();
            let root_ref = tree.get(&root_id).unwrap();
            assert_eq!(node_a_ref.data(), &a);
            assert_eq!(root_ref.data(), &a);
        }

        let node_b_id = tree.insert(node_b, AsRoot).unwrap();
        let root_id = tree.root.clone().unwrap();
        assert_eq!(node_b_id, root_id);

        {
            let node_b_ref = tree.get(&node_b_id).unwrap();
            let root_ref = tree.get(&root_id).unwrap();
            assert_eq!(node_b_ref.data(), &b);
            assert_eq!(root_ref.data(), &b);

            let node_b_child_id = node_b_ref.children().first().unwrap();
            let node_b_child_ref = tree.get(node_b_child_id).unwrap();
            assert_eq!(node_b_child_ref.data(), &a);
        }
    }

    #[test]
    fn test_root_node_id() {
        let tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();
        let root_node_id = tree.root_node_id().unwrap();

        assert_eq!(&root_id, root_node_id);
    }

    #[test]
    fn test_insert_with_parent() {
        use InsertBehavior::*;

        let a = 1;
        let b = 2;
        let r = 5;

        let mut tree = TreeBuilder::new().with_root(Node::new(r)).build();

        let node_a = Node::new(a);
        let node_b = Node::new(b);

        let root_id = tree.root.clone().unwrap();
        let node_a_id = tree.insert(node_a, UnderNode(&root_id)).unwrap();
        let node_b_id = tree.insert(node_b, UnderNode(&root_id)).unwrap();

        let node_a_ref = tree.get(&node_a_id).unwrap();
        let node_b_ref = tree.get(&node_b_id).unwrap();
        assert_eq!(node_a_ref.data(), &a);
        assert_eq!(node_b_ref.data(), &b);

        assert_eq!(node_a_ref.parent().unwrap().clone(), root_id);
        assert_eq!(node_b_ref.parent().unwrap().clone(), root_id);

        let root_node_ref = tree.get(&root_id).unwrap();
        let root_children: &Vec<NodeId> = root_node_ref.children();

        let child_1_id = root_children.first().unwrap();
        let child_2_id = root_children.get(1).unwrap();

        let child_1_ref = tree.get(child_1_id).unwrap();
        let child_2_ref = tree.get(child_2_id).unwrap();

        assert_eq!(child_1_ref.data(), &a);
        assert_eq!(child_2_ref.data(), &b);
    }

    #[test]
    fn test_remove_node_lift_children() {
        use InsertBehavior::*;
        use RemoveBehavior::*;

        let mut tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();

        let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        let node_2_id = tree.insert(Node::new(2), UnderNode(&node_1_id)).unwrap();
        let node_3_id = tree.insert(Node::new(3), UnderNode(&node_1_id)).unwrap();

        let node_1 = tree.remove_node(node_1_id.clone(), LiftChildren).unwrap();

        assert_eq!(Some(&root_id), tree.root_node_id());

        assert_eq!(node_1.data(), &1);
        assert_eq!(node_1.children().len(), 0);
        assert!(node_1.parent().is_none());
        assert!(tree.get(&node_1_id).is_err());

        let root_ref = tree.get(&root_id).unwrap();
        let node_2_ref = tree.get(&node_2_id).unwrap();
        let node_3_ref = tree.get(&node_3_id).unwrap();

        assert_eq!(node_2_ref.data(), &2);
        assert_eq!(node_3_ref.data(), &3);

        assert_eq!(node_2_ref.parent().unwrap(), &root_id);
        assert_eq!(node_3_ref.parent().unwrap(), &root_id);

        assert!(root_ref.children().contains(&node_2_id));
        assert!(root_ref.children().contains(&node_3_id));
    }

    #[test]
    fn test_remove_node_orphan_children() {
        use InsertBehavior::*;
        use RemoveBehavior::*;

        let mut tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();

        let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        let node_2_id = tree.insert(Node::new(2), UnderNode(&node_1_id)).unwrap();
        let node_3_id = tree.insert(Node::new(3), UnderNode(&node_1_id)).unwrap();

        let node_1 = tree.remove_node(node_1_id.clone(), OrphanChildren).unwrap();

        assert_eq!(Some(&root_id), tree.root_node_id());

        assert_eq!(node_1.data(), &1);
        assert_eq!(node_1.children().len(), 0);
        assert!(node_1.parent().is_none());
        assert!(tree.get(&node_1_id).is_err());

        let node_2_ref = tree.get(&node_2_id).unwrap();
        let node_3_ref = tree.get(&node_3_id).unwrap();

        assert_eq!(node_2_ref.data(), &2);
        assert_eq!(node_3_ref.data(), &3);

        assert!(node_2_ref.parent().is_none());
        assert!(node_3_ref.parent().is_none());
    }

    #[test]
    fn test_remove_root() {
        use RemoveBehavior::*;

        let mut tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();
        tree.remove_node(root_id, OrphanChildren).unwrap();
        assert_eq!(None, tree.root_node_id());

        let mut tree = TreeBuilder::new().with_root(Node::new(5)).build();

        let root_id = tree.root.clone().unwrap();
        tree.remove_node(root_id, LiftChildren).unwrap();
        assert_eq!(None, tree.root_node_id());
    }

    #[test]
    fn test_move_node_to_parent() {
        use InsertBehavior::*;
        use MoveBehavior::*;

        let mut tree = Tree::new();

        let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
        let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        let node_2_id = tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        let node_3_id = tree.insert(Node::new(3), UnderNode(&node_1_id)).unwrap();

        // Move 3 "across" the tree.
        tree.move_node(&node_3_id, ToParent(&node_2_id)).unwrap();
        assert!(tree.get(&root_id).unwrap().children().contains(&node_1_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_2_id));
        assert!(
            tree.get(&node_2_id,)
                .unwrap()
                .children()
                .contains(&node_3_id,)
        );

        // Move 3 "up" the tree.
        tree.move_node(&node_3_id, ToParent(&root_id)).unwrap();
        assert!(tree.get(&root_id).unwrap().children().contains(&node_1_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_2_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_3_id));

        // Move 3 "down" (really this is across though) the tree.
        tree.move_node(&node_3_id, ToParent(&node_1_id)).unwrap();
        assert!(tree.get(&root_id).unwrap().children().contains(&node_1_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_2_id));
        assert!(
            tree.get(&node_1_id,)
                .unwrap()
                .children()
                .contains(&node_3_id,)
        );

        // Move 1 "down" the tree.
        tree.move_node(&node_1_id, ToParent(&node_3_id)).unwrap();
        assert!(tree.get(&root_id).unwrap().children().contains(&node_2_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_3_id));
        assert!(
            tree.get(&node_3_id,)
                .unwrap()
                .children()
                .contains(&node_1_id,)
        );

        // Note: node_1 is at the lowest point in the tree before these insertions.
        let node_4_id = tree.insert(Node::new(4), UnderNode(&node_1_id)).unwrap();
        let node_5_id = tree.insert(Node::new(5), UnderNode(&node_4_id)).unwrap();

        // move 3 "down" the tree
        tree.move_node(&node_3_id, ToParent(&node_5_id)).unwrap();
        assert!(tree.get(&root_id).unwrap().children().contains(&node_2_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_1_id));
        assert!(
            tree.get(&node_1_id,)
                .unwrap()
                .children()
                .contains(&node_4_id,)
        );
        assert!(
            tree.get(&node_4_id,)
                .unwrap()
                .children()
                .contains(&node_5_id,)
        );
        assert!(
            tree.get(&node_5_id,)
                .unwrap()
                .children()
                .contains(&node_3_id,)
        );

        // move root "down" the tree
        tree.move_node(&root_id, ToParent(&node_2_id)).unwrap();
        assert!(tree.get(&node_2_id).unwrap().children().contains(&root_id));
        assert!(tree.get(&root_id).unwrap().children().contains(&node_1_id));
        assert!(
            tree.get(&node_1_id,)
                .unwrap()
                .children()
                .contains(&node_4_id,)
        );
        assert!(
            tree.get(&node_4_id,)
                .unwrap()
                .children()
                .contains(&node_5_id,)
        );
        assert!(
            tree.get(&node_5_id,)
                .unwrap()
                .children()
                .contains(&node_3_id,)
        );
        assert_eq!(tree.root_node_id(), Some(&node_2_id));
    }

    #[test]
    fn test_move_node_to_root() {
        use InsertBehavior::*;

        // test move with existing root
        {
            let mut tree = Tree::new();
            let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
            let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            let node_2_id = tree.insert(Node::new(2), UnderNode(&node_1_id)).unwrap();

            tree.move_node_to_root(&node_2_id);

            assert_eq!(tree.root_node_id(), Some(&node_2_id));
            assert!(tree.get(&node_2_id).unwrap().children().contains(&root_id));
            assert!(
                !tree
                    .get(&node_1_id,)
                    .unwrap()
                    .children()
                    .contains(&node_2_id,)
            );
        }

        // Test move with existing root and with orphan.
        {
            let mut tree = Tree::new();
            let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
            let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            let node_2_id = tree.insert(Node::new(2), UnderNode(&node_1_id)).unwrap();

            tree.remove_node_orphan_children(node_1_id);
            tree.move_node_to_root(&node_2_id);

            assert_eq!(tree.root_node_id(), Some(&node_2_id));
            assert!(tree.get(&node_2_id).unwrap().children().contains(&root_id));
            assert_eq!(tree.get(&root_id).unwrap().children().len(), 0);
        }

        // Test move without root and with orphan.
        {
            let mut tree = Tree::new();
            let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
            let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            let node_2_id = tree.insert(Node::new(2), UnderNode(&node_1_id)).unwrap();

            tree.remove_node_orphan_children(root_id);
            tree.move_node_to_root(&node_1_id);

            assert_eq!(tree.root_node_id(), Some(&node_1_id));
            assert!(
                tree.get(&node_1_id,)
                    .unwrap()
                    .children()
                    .contains(&node_2_id,)
            );
            assert_eq!(tree.get(&node_1_id).unwrap().children().len(), 1);
        }
    }

    #[test]
    fn test_find_subtree_root_below_upper_id() {
        use InsertBehavior::*;

        let mut tree = Tree::new();

        let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
        let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        let node_2_id = tree.insert(Node::new(2), UnderNode(&node_1_id)).unwrap();
        let node_3_id = tree.insert(Node::new(3), UnderNode(&node_1_id)).unwrap();
        let node_4_id = tree.insert(Node::new(4), UnderNode(&node_2_id)).unwrap();

        let sub_root = tree.find_subtree_root_between_ids(&node_1_id, &root_id);
        assert_eq!(sub_root, Some(&node_1_id));
        let sub_root = tree.find_subtree_root_between_ids(&root_id, &node_1_id); //invert for None
        assert_eq!(sub_root, None);

        let sub_root = tree.find_subtree_root_between_ids(&node_2_id, &root_id);
        assert_eq!(sub_root, Some(&node_1_id));
        let sub_root = tree.find_subtree_root_between_ids(&root_id, &node_2_id); //invert for None
        assert_eq!(sub_root, None);

        let sub_root = tree.find_subtree_root_between_ids(&node_3_id, &node_1_id);
        assert_eq!(sub_root, Some(&node_3_id));
        let sub_root = tree.find_subtree_root_between_ids(&node_1_id, &node_3_id); //invert for None
        assert_eq!(sub_root, None);

        let sub_root = tree.find_subtree_root_between_ids(&node_4_id, &root_id);
        assert_eq!(sub_root, Some(&node_1_id));
        let sub_root = tree.find_subtree_root_between_ids(&root_id, &node_4_id); //invert for None
        assert_eq!(sub_root, None);
    }

    #[test]
    fn test_tree_height() {
        use InsertBehavior::*;
        use RemoveBehavior::*;

        // Empty tree.
        let mut tree = Tree::new();
        assert_eq!(0, tree.height());

        // The tree with single root node.
        let root_id = tree.insert(Node::new(1), AsRoot).unwrap();
        assert_eq!(1, tree.height());

        // Root node with single child.
        let child_1_id = tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        assert_eq!(2, tree.height());

        // Root node with two children.
        let child_2_id = tree.insert(Node::new(3), UnderNode(&root_id)).unwrap();
        assert_eq!(2, tree.height());

        // Grandson.
        tree.insert(Node::new(4), UnderNode(&child_1_id)).unwrap();
        assert_eq!(3, tree.height());

        // Remove child_1 and grandchild.
        tree.remove_node(child_1_id, DropChildren).unwrap();
        assert_eq!(2, tree.height());

        // Remove child_2.
        tree.remove_node(child_2_id, LiftChildren).unwrap();
        assert_eq!(1, tree.height());
    }

    #[test]
    fn test_partial_eq() {
        use InsertBehavior::*;

        let mut tree = Tree::new();
        let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
        let node_1_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        tree.insert(Node::new(3), UnderNode(&node_1_id)).unwrap();

        // Ensure PartialEq doesn't work when the number of used nodes are not equal.
        {
            let mut other = Tree::new();
            let root_id = other.insert(Node::new(0), AsRoot).unwrap();
            other.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(2), UnderNode(&root_id)).unwrap();
            assert_ne!(tree, other);
        }

        // Ensure PartialEq doesn't work when the data is not equal.
        {
            let mut other = Tree::new();
            let root_id = other.insert(Node::new(0), AsRoot).unwrap();
            let id = other.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(2), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(4), UnderNode(&id)).unwrap();
            assert_ne!(tree, other);
        }

        // Ensure PartialEq doesn't work when the parents aren't equal.
        {
            let mut other = Tree::new();
            let root_id = other.insert(Node::new(0), AsRoot).unwrap();
            other.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            let id = other.insert(Node::new(2), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(3), UnderNode(&id)).unwrap();
            assert_ne!(tree, other);
        }

        // Ensure PartialEq works even if the number of free spots in Tree.
        // Node is different.
        {
            let mut other = Tree::new();
            let root_id = other.insert(Node::new(0), AsRoot).unwrap();
            let id = other.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(2), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(3), UnderNode(&id)).unwrap();
            let to_delete = other.insert(Node::new(42), UnderNode(&root_id)).unwrap();
            other.take_node(to_delete);
            assert_ne!(
                tree.nodes.iter().filter(|x| x.is_none()).count(),
                other.nodes.iter().filter(|x| x.is_none()).count()
            );
            assert_eq!(tree, other);
        }

        // Ensure PartialEq doesn't work when the Node's index are different.
        {
            let mut other = Tree::new();
            let root_id = other.insert(Node::new(0), AsRoot).unwrap();
            let to_delete = other.insert(Node::new(42), UnderNode(&root_id)).unwrap();
            let id = other.insert(Node::new(1), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(2), UnderNode(&root_id)).unwrap();
            other.insert(Node::new(3), UnderNode(&id)).unwrap();
            other.take_node(to_delete);
            assert_ne!(tree, other);
        }
    }
}
