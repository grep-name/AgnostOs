use noto_sans_mono_bitmap::{FontWeight, RasterHeight};

pub(crate) const FONT_WEIGHT: FontWeight = FontWeight::Regular;
pub(crate) const FONT_HEIGHT: RasterHeight = RasterHeight::Size16;

/// Module that contains the code for our custom allocator.
pub mod allocator;

/// Module that contains the code for rendering things to the screen after exiting uefi boot
/// services. It usses framebuffer to write the bytes directly
pub mod graphics;

/// Module that contains the code for rendering things to the screen when in uefi. It usses gop.
pub mod uefi_graphics;

pub mod kprintln;

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
