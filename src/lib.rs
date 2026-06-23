use std::sync::atomic::AtomicUsize;

use noto_sans_mono_bitmap::{FontWeight, RasterHeight};

pub(crate) const FONT_WEIGHT: FontWeight = FontWeight::Regular;
pub(crate) const FONT_HEIGHT: RasterHeight = RasterHeight::Size16;

pub static HEAP_START: AtomicUsize = AtomicUsize::new(0);
pub static HEAP_SIZE: AtomicUsize = AtomicUsize::new(0);

/// Module that contains the code for our custom allocator.
pub mod allocator;

/// Module that contains the code for rendering things to the screen after exiting uefi boot
/// services. It usses framebuffer to write the bytes directly
pub mod graphics;

/// Module that contains the code for rendering things to the screen when in uefi. It usses gop.
pub mod uefi_graphics;

/// Module that contains the code for printing text to the screen same way println! does
pub mod console;

/// Module that contains the code for using Colors
pub mod color;

/// Module that contains the code for handling keyboard.
pub mod keyboard;

/// Module that contains the code for shell.
pub mod shell;
