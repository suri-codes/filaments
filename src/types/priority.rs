use std::fmt::Display;

use color_eyre::eyre::eyre;
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

impl TryFrom<&str> for Priority {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().chars().next() {
            Some('a') => Ok(Self {
                field1: PriorityDTO::Asap,
            }),
            Some('h') => Ok(Self {
                field1: PriorityDTO::High,
            }),
            Some('m') => Ok(Self {
                field1: PriorityDTO::Medium,
            }),
            Some('l') => Ok(Self {
                field1: PriorityDTO::Low,
            }),
            Some('f') => Ok(Self {
                field1: PriorityDTO::Far,
            }),
            _ => Err(eyre!("Invalid Priority!")),
        }
    }
}
