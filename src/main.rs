#![no_main]
#![no_std]

use core::{any, time::Duration};

extern crate alloc;

use agnostos::{
    allocator::AgnostosAllocator,
    color,
    graphics::{self, Framebuffer},
    keyboard, kprint, kprintln,
};
use alloc::{format, string::String};
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
    agnostos::kprintln::init(&fb);

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

    //    let s = format!(
    //        "heap_start: {} \n heap_end: {} \n heap_size: {}mb",
    //        heap_start,
    //        heap_start + heap_size,
    //        heap_size / (1024 * 1024)
    //    );
    //
    //    let msg = stress_test();
    //
    agnostos::graphics::clear_background(&fb, color::BLACK);
    //
    //    kprintln!("{}", &msg);
    //    kprintln!("{}", &s);
    //
    //    kprintln!("------------------------------------------");
    //
    //    kprintln!("comparing both the versions of rendering text");
    //    kprintln!("survived 10000 allocs!");
    //

    let mut line = String::new();

    kprint!("> ");
    loop {
        if let Some(code) = keyboard::read_scan_code_if_available() {
            if let Some(ch) = keyboard::scancode_to_ascii(code) {
                kprintln::erase_cursor();
                match ch {
                    '\n' => {
                        kprintln!();
                        run_command(&line);
                        line.clear();
                        kprint!("> ");
                    }

                    '\u{8}' => {
                        if line.pop().is_some() {
                            kprintln::backspace();
                        }
                    }

                    c => {
                        line.push(c);
                        kprint!("{}", c);
                    }
                }
                kprintln::draw_cursor(); // redraw cursor at new position
            }
        }
    }
}

fn run_command(command: &str) {
    let command = command.trim();

    match command {
        "help" => kprintln!("Commands: help, clear, about"),
        "about" => kprintln!("AgnostOs v0.1 - written in Rust \n github.com/grep-name/agnostos"),
        "" => {}
        "clear" => agnostos::kprintln::reset(),
        _ => {
            kprintln!("Unknown command");
        }
    }
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
