use rand::RngExt;
use rgb::RGB8;
use sea_orm::DeriveValueType;
use std::fmt::{Debug, Display};

/// Color type
///
/// We store it as a u32, but its actually 00000000rrrrrrrrggggggggbbbbbbbb
#[derive(Clone, Copy, PartialEq, Eq, DeriveValueType)]
pub struct Color(u32);

impl Color {
    /// create a new color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    /// to convert it into a rbg8 type
    pub fn to_rgb8(self) -> RGB8 {
        RGB8 {
            r: ((self.0 >> 16) & 0xFF) as u8,
            g: ((self.0 >> 8) & 0xFF) as u8,
            b: (self.0 & 0xFF) as u8,
        }
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rgb = self.to_rgb8();
        write!(f, "Color(#{:02X}{:02X}{:02X})", rgb.r, rgb.g, rgb.b)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rgb = self.to_rgb8();
        write!(f, "#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b)
    }
}

impl From<RGB8> for Color {
    fn from(c: RGB8) -> Self {
        Self::new(c.r, c.g, c.b)
    }
}

impl From<Color> for RGB8 {
    fn from(c: Color) -> Self {
        c.to_rgb8()
    }
}

impl Default for Color {
    fn default() -> Self {
        let mut rng = rand::rng();
        let r = rng.random_range(0..=255);
        let g = rng.random_range(0..=255);
        let b = rng.random_range(0..=255);

        Self::new(r, g, b)
    }
}
