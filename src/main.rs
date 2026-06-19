#![no_main]
#![no_std]

use core::time::Duration;

extern crate alloc;

use alloc::{format, string::String};
use something::{allocator::SomethingAllocator, graphics::Framebuffer, kprintln};
use uefi::{
    boot::{MemoryType, OpenProtocolAttributes, OpenProtocolParams},
    mem::memory_map::MemoryMap,
    prelude::*,
    proto::console::gop::GraphicsOutput,
};

#[global_allocator]
static ALLOCATOR: SomethingAllocator = SomethingAllocator::new();

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
    something::kprintln::init(&fb);

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

    // Giving the allocator the pointer to heap
    ALLOCATOR.init(heap_start, heap_size);

    let s = format!(
        "heap_start: {} \n heap_end: {} \n heap_size: {}mb",
        heap_start,
        heap_start + heap_size,
        heap_size / (1024 * 1024)
    );

    let msg = stress_test();

    something::graphics::clear_background(&fb, [0, 0, 0]);

    kprintln!("{}", &msg);
    kprintln!("{}", &s);

    kprintln!("------------------------------------------");

    kprintln!("comparing both the versions of rendering text");

    something::graphics::draw_text(&fb, "survived 10000 allocs!", (100, 200), [255, 255, 255]);
    kprintln!("survived 10000 allocs!");

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

/// Allocates 10000 vectors to stress test our allocator implementation
fn stress_test() -> String {
    // Stress testing
    let addr1;
    let addr2;

    {
        let v: alloc::vec::Vec<u8> = alloc::vec![1, 2, 3];
        addr1 = v.as_ptr() as usize;
    }

    {
        let v2: alloc::vec::Vec<u8> = alloc::vec![4, 5, 6];
        addr2 = v2.as_ptr() as usize;
    }

    for _ in 0..10000 {
        let _: alloc::vec::Vec<u8> = alloc::vec![0u8; 1024];
    }

    // if dealloc works, addr1 and addr2 should be the same (or very close)
    let msg = format!(
        "addr1: {:#x} addr2: {:#x} same: {}",
        addr1,
        addr2,
        addr1 == addr2
    );

    return msg;
}
