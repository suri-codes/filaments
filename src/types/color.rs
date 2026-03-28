use dto::ColorDTO;

/// Agnostic Color type,
/// internally represented as rgb
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Color(ColorDTO);

impl From<ColorDTO> for Color {
    fn from(value: ColorDTO) -> Self {
        Self(value)
    }
}
