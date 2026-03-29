use serde::{Deserialize, Serialize};

use crate::types::ZettelId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub source: ZettelId,
    pub dest: ZettelId,
}

impl Link {
    pub fn new(source: impl Into<ZettelId>, dest: impl Into<ZettelId>) -> Self {
        Self {
            source: source.into(),
            dest: dest.into(),
        }
    }
}
