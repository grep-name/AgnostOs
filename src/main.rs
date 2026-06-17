#![no_main]
#![no_std]

use core::time::Duration;

extern crate alloc;

use alloc::format;
use something::{allocator::SomethingAllocator, graphics::Framebuffer};
use uefi::{
    boot::{MemoryType, OpenProtocolAttributes, OpenProtocolParams},
    mem::memory_map::MemoryMap,
    prelude::*,
    proto::console::gop::{GraphicsOutput, PixelFormat},
};

#[global_allocator]
static ALLOCATOR: SomethingAllocator = SomethingAllocator::new();

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    uefi::println!("Hello From RUST");

    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .expect("missing graphics output protocol");

    let gop = unsafe {
        &mut boot::open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle: gop_handle,
                agent: boot::image_handle(),
                controller: None,
            },
            // For this test, don't open in exclusive mode. That
            // would break the connection between stdout and the
            // video console.
            OpenProtocolAttributes::GetProtocol,
        )
        .expect("failed to open Graphics Output Protocol")
    };

    set_graphics_mode(gop);

    let fb = Framebuffer::new(gop);

    uefi::println!("Exiting boot services in 3 seconds...");

    let dr = Duration::from_millis(3000);
    boot::stall(dr);

    let memory_map = unsafe { boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };

    let mut heap_start = 0usize;
    let mut heap_size = 0usize;

    for descriptor in memory_map.entries() {
        // Free usable memory
        if descriptor.ty == MemoryType::CONVENTIONAL {
            let size = descriptor.page_count as usize * 4096;

            if size > heap_size {
                heap_start = descriptor.phys_start as usize;
                heap_size = size;
            }
        }
    }

    ALLOCATOR.init(heap_start, heap_size);

    let s = format!(
        "heap_start: {} \n heap_end: {} \n heap_size: {}mb",
        heap_start,
        heap_start + heap_size,
        heap_size / (1024 * 1024)
    );

    something::graphics::clear_background(&fb, [255, 255, 255]);
    something::graphics::draw_text(&fb, &s, (100, 100), [0, 0, 0], 1);

    loop {}
}

fn set_graphics_mode(gop: &mut GraphicsOutput) {
    let mode = gop
        .modes()
        .find(|mode| {
            let info = mode.info();
            info.resolution() == (1024, 768)
        })
        .unwrap();

    gop.set_mode(&mode).expect("Failed to set graphics mode");
}
