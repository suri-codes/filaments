use super::NodeId;

/// Describes the possible behaviors of the `Tree::insert` method.
pub enum InsertBehavior<'a> {
    /// Insert the `Node` as the root of the tree.
    ///
    /// If there is already a root `Node` in the tree, then that `Node` will
    /// be set as the first child as the new root `Node`.
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let root_node = Node::new(1);
    ///
    /// tree.insert(root_node, AsRoot).unwrap();
    ///
    /// ```
    AsRoot,

    /// Inserts the `Node` under the `Node` that has the provided `NodeId`.
    ///
    /// Note: Adds the new `Node` to the end of its children.
    ///
    /// # Returns
    /// `Result` containing the `NodeId` of the child that was added or a `NodeIdError`
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    ///
    /// let root_node = Node::new(1);
    /// let child_node = Node::new(2);
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    /// let mut root_id = tree.insert(root_node, AsRoot).unwrap();
    ///
    /// tree.insert(child_node, UnderNode(&root_id)).unwrap();
    ///
    /// ```
    UnderNode(&'a NodeId),
}

pub enum RemoveBehavior {
    /// The entire subtree of the `Node` being removed will be
    /// dropped from the tree, effectively meaning that all children
    /// will be dropped recursively.
    ///
    /// Those `Node`'s will no longer exist and cannot be accessed even
    /// if you hold a `NodeId` for them, so use this behavior with caution.
    ///
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    /// use sakura::RemoveBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let child_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// let grandchild_id = tree.insert(Node::new(2), UnderNode(&child_id)).unwrap();
    ///
    /// let child = tree.remove_node(child_id, DropChildren).ok().unwrap();
    ///
    /// assert!(tree.get(&grandchild_id).is_err());
    /// assert_eq!(tree.get(&root_id).unwrap().children().len(), 0);
    /// assert_eq!(child.children().len(), 0);
    /// assert_eq!(child.parent(), None);
    /// ```
    ///
    DropChildren,

    ///
    ///
    /// If the removed `Node` (say `A`) has a parent `A'`, then `A'` will
    /// become the parent of `A`'s children.
    ///
    /// If `A` doesn't have a parent, then this behaves exactly like
    /// `RemoveBehavior::OrphanChildren`.
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    /// use sakura::RemoveBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let child_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// let grandchild_id = tree.insert(Node::new(2), UnderNode(&child_id)).unwrap();
    ///
    /// let child = tree.remove_node(child_id, LiftChildren).ok().unwrap();
    ///
    /// assert!(tree.get(&grandchild_id).is_ok());
    /// assert!(tree.get(&root_id).unwrap().children().contains(&grandchild_id));
    /// assert_eq!(child.children().len(), 0);
    /// assert_eq!(child.parent(), None);
    /// ```
    ///
    LiftChildren,

    /// All children will have their parent references cleared. Nothing
    /// will point to them, but they will still exist in the tree.
    /// Those `Node`s can still be accessed if you still have their
    /// `NodeId`'s.
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    /// use sakura::RemoveBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
    /// let child_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    /// let grandchild_id = tree.insert(Node::new(2), UnderNode(&child_id)).unwrap();
    ///
    /// let child = tree.remove_node(child_id, OrphanChildren).ok().unwrap();
    ///
    /// assert!(tree.get(&grandchild_id).is_ok());
    /// assert_eq!(tree.get(&root_id).unwrap().children().len(), 0);
    /// assert_eq!(child.children().len(), 0);
    /// assert_eq!(child.parent(), None);
    /// ```
    ///
    OrphanChildren,
}

pub enum MoveBehavior<'a> {
    /// Sets the `Node` as the new root `Node`, while having all their children
    /// travel with them.
    ///
    /// If there is already a root `Node` in place, it will be attached as the
    /// last child of the new root `Node`.
    ///
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    /// use sakura::MoveBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(1), AsRoot).unwrap();
    /// let child_id = tree.insert(Node::new(2),  UnderNode(&root_id)).unwrap();
    /// let grandchild_id = tree.insert(Node::new(3), UnderNode(&child_id)).unwrap();
    ///
    /// tree.move_node(&grandchild_id, ToRoot).unwrap();
    ///
    /// assert_eq!(tree.root_node_id(), Some(&grandchild_id));
    /// assert!(tree.get(&grandchild_id).unwrap().children().contains(&root_id));
    /// assert!(!tree.get(&child_id).unwrap().children().contains(&grandchild_id));
    /// ```
    ///
    ToRoot,

    ///
    /// Moves a `Node` in the `Tree` to a new parent, while leaving
    /// all children in their place.
    ///
    /// If the new parent (lets say `A'`) is a descendant of the `Node` being
    /// moved (`A`), then the direct child of `A` on the path from `A` to
    /// `A'` will be shifted upwards to take the place of its parent (`A`).
    /// All other children of `A` will be left alone, so they will travel
    /// with `A`.
    ///
    /// NOTE: During the shift-up part of the above scenario, the `Node` being
    /// shifted up will always be added as the last child of its new parent.
    /// ```
    /// use sakura::*;
    /// use sakura::InsertBehavior::*;
    /// use sakura::MoveBehavior::*;
    ///
    /// let mut tree: Tree<i32> = Tree::new();
    ///
    /// let root_id = tree.insert(Node::new(1), AsRoot).ok().unwrap();
    /// let first_child_id = tree.insert(Node::new(2),  UnderNode(&root_id)).unwrap();
    /// let second_child_id = tree.insert(Node::new(3), UnderNode(&root_id)).unwrap();
    /// let grandchild_id = tree.insert(Node::new(4), UnderNode(&first_child_id)).unwrap();
    ///
    /// tree.move_node(&grandchild_id, ToParent(&second_child_id)).unwrap();
    ///
    /// assert!(!tree.get(&first_child_id).unwrap().children().contains(&grandchild_id));
    /// assert!(tree.get(&second_child_id).unwrap().children().contains(&grandchild_id));
    /// ```
    ///
    ToParent(&'a NodeId),
}
