use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::NodeId;

#[derive(Debug, Serialize, Deserialize, Reconcile, Hydrate, Ord, Eq, PartialOrd)]
pub struct Node<T> {
    pub(crate) data: T,
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
}

impl<T> PartialEq for Node<T>
where
    T: PartialEq,
{
    // We only care if node data is equivalent.
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Node<T> {
    /// Creates a new `Node` with the provided data
    ///
    /// ```    
    /// use sakura::Node;
    ///
    /// let _one: Node<i32> = Node::new(1);
    /// ```
    ///
    #[allow(clippy::use_self)]
    pub const fn new(data: T) -> Node<T> {
        Self {
            parent: None,
            data,
            children: vec![],
        }
    }

    /// Returns a reference to the data inside the `Node`
    ///
    /// ```
    /// use sakura::Node;
    ///
    /// let x = 10;
    /// let node: Node<i32> = Node::new(x);
    /// # assert_eq!(node.data(), &10);
    /// ```
    pub const fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the data inside the `Node`
    ///
    /// ```
    /// use sakura::Node;
    ///
    /// let x = 10;
    /// let mut node: Node<i32> = Node::new(x);
    /// # assert_eq!(node.data_mut(), &mut 10);
    /// ```
    pub const fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Replaces this `Node`s data with the provided data
    ///
    /// Returns the data previously in the node
    ///
    /// ```
    /// use sakura::Node;
    ///
    /// let x = 10;
    /// let mut y = 15;
    ///
    /// let mut node_x: Node<i32> = Node::new(x);
    /// let replaced_x = node_x.replace_data(y);
    /// let new_x = node_x.data();
    ///
    /// # assert_eq!(*new_x, y);
    /// # assert_eq!(replaced_x, x);
    /// ```
    pub const fn replace_data(&mut self, mut data: T) -> T {
        ::std::mem::swap(&mut data, self.data_mut());
        data
    }

    /// Returns the parent of this `Node`, if it has one.
    ///
    /// ```
    /// use sakura::Node;
    ///
    /// let node: Node<i32> = Node::new(1);
    /// # assert_eq!(node.parent(), None);
    /// ```
    pub const fn parent(&self) -> Option<&NodeId> {
        self.parent.as_ref()
    }

    /// Returns the children of this `Node`
    ///
    /// ```
    /// use sakura::Node;
    ///
    /// let node: Node<i32> = Node::new(0);
    /// # assert_eq!(node.children().len(), 0);
    /// ```
    pub const fn children(&self) -> &Vec<NodeId> {
        &self.children
    }

    pub(crate) const fn children_mut(&mut self) -> &mut Vec<NodeId> {
        &mut self.children
    }

    pub(crate) const fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    pub(crate) fn add_child(&mut self, child: NodeId) {
        self.children.push(child);
    }

    pub(crate) fn set_children(&mut self, children: Vec<NodeId>) {
        self.children = children;
    }

    pub(crate) fn take_children(&mut self) -> Vec<NodeId> {
        use std::mem;

        let mut empty = Vec::with_capacity(0);
        mem::swap(&mut self.children, &mut empty);
        // post-swap this holds children
        empty
    }
}

#[cfg(test)]
mod node_tests {

    use super::super::NodeId;
    use super::Node;

    #[test]
    fn test_new() {
        let node = Node::new(10);
        assert_eq!(node.children.capacity(), 0);
    }

    #[test]
    fn test_data() {
        let data = 0;
        let node = Node::new(data);

        assert_eq!(node.data(), &data);
    }

    #[test]
    fn test_data_mut() {
        let mut data = 0;
        let mut node = Node::new(data);

        assert_eq!(node.data_mut(), &mut data);
    }

    #[test]
    fn test_parent() {
        let mut node = Node::new(0);

        assert!(node.parent().is_none());

        let parent_id: NodeId = NodeId { index: 100 };

        node.set_parent(Some(parent_id.clone()));

        assert_eq!(node.parent, Some(parent_id));
    }

    #[test]
    fn test_children() {
        let mut node = Node::new(0);
        assert!(node.children.is_empty());

        let child_id: NodeId = NodeId { index: 1 };

        node.add_child(child_id.clone());

        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children.first().unwrap(), &child_id);
    }

    #[test]
    fn test_partial_eq() {
        let node1 = Node::new(32);
        let node2 = Node::new(32);
        let node3 = Node::new(64);

        assert_eq!(node1, node2);

        assert_ne!(node1, node3);
        assert_ne!(node2, node3);
    }
}
