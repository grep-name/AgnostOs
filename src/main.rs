#![no_main]
#![no_std]

use core::time::Duration;

use something::graphics::Framebuffer;
use uefi::{
    boot::{MemoryType, OpenProtocolAttributes, OpenProtocolParams},
    prelude::*,
    proto::console::gop::{GraphicsOutput, PixelFormat},
};

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

    something::uefi_graphics::clear_background(gop, [20, 255, 50]);
    something::uefi_graphics::draw_rec(gop, (20, 20), (30, 30), [255, 255, 255]);
    something::uefi_graphics::draw_line(gop, (20, 20), (30, 30), [255, 255, 255]);
    something::uefi_graphics::draw_circle(gop, 40, (20, 20), [255, 255, 255]);
    something::uefi_graphics::draw_text(gop, "something something", (100, 100), [255, 255, 255], 1);

    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    let stride = mode_info.stride();
    let is_bgr = matches!(mode_info.pixel_format(), PixelFormat::Bgr);
    let ptr = gop.frame_buffer().as_mut_ptr();

    let fb = Framebuffer {
        ptr,
        width,
        height,
        stride,
        is_bgr,
    };

    uefi::println!("Framebuffer: {:?}", fb);
    uefi::println!("Exiting boot services in 3 seconds...");

    let dr = Duration::from_millis(3000);
    boot::stall(dr);

    let _memory_map = unsafe { boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };

    something::graphics::clear_background(&fb, [255, 255, 255]);

    something::graphics::draw_rec(&fb, (20, 20), (30, 30), [0, 0, 0]);
    something::graphics::draw_line(&fb, (20, 20), (30, 30), [0, 0, 0]);
    something::graphics::draw_circle(&fb, 40, (20, 20), [0, 0, 0]);
    something::graphics::draw_text(&fb, "something something", (100, 100), [0, 0, 0], 1);

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
