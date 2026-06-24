#![no_main]
#![no_std]

use core::time::Duration;

extern crate alloc;

use agnostos::{
    allocator::AgnostosAllocator, console, graphics::Framebuffer, shell, uefi_graphics,
};

use uefi::prelude::*;

#[global_allocator]
static ALLOCATOR: AgnostosAllocator = AgnostosAllocator::new();

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let mut gop = uefi_graphics::init_gop();
    let fb = Framebuffer::new(&mut gop);

    console::init(&fb);
    uefi::println!("Exiting boot services in 1 seconds...");

    boot::stall(Duration::from_millis(1000));

    ALLOCATOR.init();
    shell::init(&fb)
}
