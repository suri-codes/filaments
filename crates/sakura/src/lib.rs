#![forbid(unsafe_code)]
//! A purpose-built library for CASE, to hold the internal tree-like
//! data structure that holds tasks and groups. Additionally, this
//! data structure is compatible with `AutoMerge`, via `AutoSurgeon`
//!
//! TODO: add example usage

use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

mod behaviors;
mod error;
mod iterators;
mod node;
mod tree;

pub use node::Node;

pub use tree::Tree;
pub use tree::TreeBuilder;

pub use behaviors::InsertBehavior;
pub use behaviors::MoveBehavior;
pub use behaviors::RemoveBehavior;

pub use iterators::Ancestors;
pub use iterators::Children;
pub use iterators::ChildrenIds;
pub use iterators::PreOrderTraversal;
pub use iterators::PreOrderTraversalIds;

pub use error::NodeIdError;

/// A Node Id
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reconcile, Hydrate,
)]
pub struct NodeId {
    index: u32,
}

impl NodeId {
    // This is okay since we are practically never reaching 2^32.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const fn new(index: usize) -> Self {
        Self {
            index: index as u32,
        }
    }
}
