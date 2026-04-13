use std::fmt::Display;

use dto::PriorityDTO;

/// An Enum for the various `Priority` levels
/// for `Task`s and `Group`s
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority {
    field1: PriorityDTO,
}

impl From<PriorityDTO> for Priority {
    fn from(value: PriorityDTO) -> Self {
        Self { field1: value }
    }
}

impl From<Priority> for PriorityDTO {
    fn from(value: Priority) -> Self {
        value.field1
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.field1)
    }
}
