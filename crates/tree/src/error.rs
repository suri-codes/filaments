use std::{error::Error, fmt::Display};

/// Enum for all possible `NodeId` errors that could happen.
#[derive(Debug, PartialEq, Eq)]
pub enum NodeIdError {
    /// Occurs when a `NodeId` is used on a `Tree` after the corresponding
    /// `Node` has been removed.
    NodeIdNoLongerValid,
}

impl NodeIdError {
    const fn to_string(&self) -> &str {
        match *self {
            Self::NodeIdNoLongerValid => {
                "The given NodeId is no longer valid. The Node in question has been removed."
            }
        }
    }
}

impl Display for NodeIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeIdError: {}", self.to_string())
    }
}

impl Error for NodeIdError {}
