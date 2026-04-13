use std::fmt::Display;

use dto::ColorDTO;
use ratatui::style::Color as RatColor;

/// Agnostic Color type,
/// internally represented as rgb
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Default)]
pub struct Color(ColorDTO);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rgb = self.0.to_rgb8();
        write!(f, "#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b)
    }
}

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
