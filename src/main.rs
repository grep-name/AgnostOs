#![no_main]
#![no_std]

use core::time::Duration;

extern crate alloc;

use agnostos::{HEAP_SIZE, HEAP_START, shell};
use agnostos::{allocator::AgnostosAllocator, color, graphics::Framebuffer};
use core::sync::atomic::Ordering;

use uefi::{
    boot::{MemoryType, OpenProtocolAttributes, OpenProtocolParams},
    mem::memory_map::MemoryMap,
    prelude::*,
    proto::console::gop::GraphicsOutput,
};

#[global_allocator]
static ALLOCATOR: AgnostosAllocator = AgnostosAllocator::new();

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    uefi::println!("Initialized UEFI environment and essential features.");

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
    agnostos::console::init(&fb);

    uefi::println!("Exiting boot services in 1 seconds...");

    let dr = Duration::from_millis(1000);
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

    // Giving the allocator the pointer to heap
    HEAP_START.store(heap_start, Ordering::Relaxed);
    HEAP_SIZE.store(heap_size, Ordering::Relaxed);

    ALLOCATOR.init(heap_start, heap_size);

    agnostos::graphics::clear_background(&fb, color::BLACK);

    return shell::init();
}

fn set_graphics_mode(gop: &mut GraphicsOutput) {
    let mode = gop
        .modes()
        .max_by_key(|mode| {
            let (w, h) = mode.info().resolution();
            w * h
        })
        .expect("no graphics modes available");
    gop.set_mode(&mode).expect("Failed to set graphics mode");
}
