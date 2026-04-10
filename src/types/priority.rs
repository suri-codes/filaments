use dto::PriorityDTO;

/// An Enum for the various `Priority` levels
/// for `Task`s and `Group`s
#[derive(Debug, Clone, Default)]
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
