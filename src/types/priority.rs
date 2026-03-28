use dto::PriorityDTO;

/// An Enum for the various `Priority` levels
/// for `Task`s and `Group`s
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Priority(PriorityDTO);

impl From<PriorityDTO> for Priority {
    fn from(value: PriorityDTO) -> Self {
        Self(value)
    }
}
