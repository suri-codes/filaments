use dto::ColorDTO;
use ratatui::style::Color as RatColor;

/// Agnostic Color type,
/// internally represented as rgb
#[derive(Debug, Copy, Clone, Default)]
pub struct Color(ColorDTO);

impl From<ColorDTO> for Color {
    fn from(value: ColorDTO) -> Self {
        Self(value)
    }
}

impl From<Color> for RatColor {
    fn from(value: Color) -> Self {
        let rgb = value.0.to_rgb8();

        Self::Rgb(rgb.r, rgb.g, rgb.b)
    }
}
