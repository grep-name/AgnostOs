/// Represents an RGB color.
///
/// Each channel uses an 8-bit value in the range `0..=255`.
pub struct Color {
    /// Red component (`0..=255`).
    pub r: u8,
    /// Green component (`0..=255`).
    pub g: u8,
    /// Blue component (`0..=255`).
    pub b: u8,
}

pub const WHITE: Color = Color::new(255, 255, 255);
pub const BLACK: Color = Color::new(0, 0, 0);

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
