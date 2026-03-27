use std::{collections::VecDeque, slice::Iter};

use crate::{Node, NodeId, Tree};

/// An `Iterator` over the children of a `Node`.
///
/// Iterates over the child `Node`s of a given `Node` in the `Tree`.
/// Each call to `next` will return an immutable
/// reference to the next child `Node`.
pub struct Children<'a, T: 'a> {
    tree: &'a Tree<T>,
    child_ids: Iter<'a, NodeId>,
}

impl<'a, T> Children<'a, T> {
    // we actually want to
    #[allow(clippy::use_self)]
    pub(crate) fn new(tree: &'a Tree<T>, node_id: &NodeId) -> Children<'a, T> {
        Children {
            tree,
            child_ids: tree
                .get(node_id)
                .expect(
                    "Function is crate specific, expecting to only be used
                with a valid node_id",
                )
                .children()
                .as_slice()
                .iter(),
        }
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.child_ids
            .next()
            .and_then(|child_id| self.tree.get(child_id).ok())
    }
}

impl<T> Clone for Children<'_, T> {
    fn clone(&self) -> Self {
        Children {
            tree: self.tree,
            child_ids: self.child_ids.clone(),
        }
    }
}

/// An `Iterator` over the children of a `Node`.
///
/// Iterates over the child `NodeId`s of a given `NodeId` in the `Tree`.
/// Each call to `next` will return an immutable
/// reference to the next child `NodeId`.
pub struct ChildrenIds<'a> {
    child_ids: Iter<'a, NodeId>,
}

impl<'a> ChildrenIds<'a> {
    #[allow(clippy::use_self)]
    pub(crate) fn new<T>(tree: &'a Tree<T>, node_id: &NodeId) -> ChildrenIds<'a> {
        ChildrenIds {
            child_ids: tree
                .get(node_id)
                .expect(
                    "Function is crate specific, expecting to only be used
                with a valid node_id",
                )
                .children()
                .as_slice()
                .iter(),
        }
    }
}

impl<'a> Iterator for ChildrenIds<'a> {
    type Item = &'a NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.child_ids.next()
    }
}

/// An `Iterator` over the ancestors of a `Node`.
///
/// Iterates over the ancestor `Node`s of given `Node` in the `Tree`.
/// Each call to `next` will return an immutable reference to the next
/// `Node` up the `Tree`.
pub struct Ancestors<'a, T: 'a> {
    tree: &'a Tree<T>,
    node_id: Option<NodeId>,
}

impl<'a, T> Ancestors<'a, T> {
    #[allow(clippy::use_self)]
    pub(crate) const fn new(tree: &'a Tree<T>, node_id: NodeId) -> Ancestors<'a, T> {
        Ancestors {
            tree,
            node_id: Some(node_id),
        }
    }
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<&'a Node<T>> {
        self.node_id
            .take()
            .and_then(|current_id| self.tree.get(&current_id).ok())
            .and_then(|node_ref| node_ref.parent())
            .and_then(|parent_id| {
                self.node_id = Some(parent_id.clone());
                self.tree.get(parent_id).ok()
            })
    }
}

impl<T> Clone for Ancestors<'_, T> {
    fn clone(&self) -> Self {
        Ancestors {
            tree: self.tree,
            node_id: self.node_id.clone(),
        }
    }
}

/// An `Iterator` over the ancestors of a `Node`.
///
/// Iterates over `NodeId`s instead of over `Node`s themselves.
pub struct AncestorsIds<'a, T: 'a> {
    tree: &'a Tree<T>,
    node_id: Option<NodeId>,
}

impl<'a, T> AncestorsIds<'a, T> {
    #[allow(clippy::use_self)]
    pub(crate) const fn new(tree: &'a Tree<T>, node_id: NodeId) -> AncestorsIds<'a, T> {
        AncestorsIds {
            tree,
            node_id: Some(node_id),
        }
    }
}

impl<'a, T> Iterator for AncestorsIds<'a, T> {
    type Item = &'a NodeId;

    fn next(&mut self) -> Option<&'a NodeId> {
        self.node_id
            .take()
            .and_then(|current_id| self.tree.get(&current_id).ok())
            .and_then(|node_ref| node_ref.parent())
            .inspect(|parent_id| {
                self.node_id = Some((*parent_id).clone());
            })
    }
}

impl<T> Clone for AncestorsIds<'_, T> {
    fn clone(&self) -> Self {
        AncestorsIds {
            tree: self.tree,
            node_id: self.node_id.clone(),
        }
    }
}

/// An iterator over the subtree relative to a given `Node`.
///
/// Each call to `next` will return an immutable reference to the
/// next `Node` in Pre-Order Traversal order.
pub struct PreOrderTraversal<'a, T: 'a> {
    tree: &'a Tree<T>,
    data: VecDeque<NodeId>,
}

impl<'a, T> PreOrderTraversal<'a, T> {
    #[allow(clippy::use_self)]
    pub(crate) fn new(tree: &'a Tree<T>, node_id: NodeId) -> PreOrderTraversal<'a, T> {
        let mut data = VecDeque::with_capacity(tree.capacity());
        data.push_front(node_id);

        PreOrderTraversal { tree, data }
    }
}

impl<'a, T> Iterator for PreOrderTraversal<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.data
            .pop_front()
            .and_then(|node_id| self.tree.get(&node_id).ok())
            .inspect(|node_ref| {
                for child_id in node_ref.children().iter().rev() {
                    self.data.push_front(child_id.clone());
                }
            })
    }
}

impl<T> Clone for PreOrderTraversal<'_, T> {
    fn clone(&self) -> Self {
        PreOrderTraversal {
            tree: self.tree,
            data: self.data.clone(),
        }
    }
}

/// An Iterator over the subtree relative to a given `Node`.
///
/// Each call to `next` will return an immutable reference to the
/// next `NodeId` in Pre-Order Traversal order.
///
pub struct PreOrderTraversalIds<'a, T: 'a> {
    tree: &'a Tree<T>,
    data: VecDeque<NodeId>,
}

impl<'a, T> PreOrderTraversalIds<'a, T> {
    #[allow(clippy::use_self)]
    pub(crate) fn new(tree: &'a Tree<T>, node_id: NodeId) -> PreOrderTraversalIds<'a, T> {
        // Over allocating, but all at once instead of resizing and reallocating as we go.
        let mut data = VecDeque::with_capacity(tree.capacity());

        data.push_front(node_id);

        PreOrderTraversalIds { tree, data }
    }
}

impl<T> Iterator for PreOrderTraversalIds<'_, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        self.data.pop_front().and_then(|node_id| {
            self.tree.get(&node_id).ok().map(|node_ref| {
                // prepend child_ids
                for child_id in node_ref.children().iter().rev() {
                    self.data.push_front(child_id.clone());
                }

                node_id
            })
        })
    }
}

impl<T> Clone for PreOrderTraversalIds<'_, T> {
    fn clone(&self) -> Self {
        PreOrderTraversalIds {
            tree: self.tree,
            data: self.data.clone(),
        }
    }
}
