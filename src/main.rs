#![no_main]
#![no_std]

use uefi::{
    boot::{OpenProtocolAttributes, OpenProtocolParams},
    prelude::*,
    proto::console::gop::GraphicsOutput,
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

    uefi::println!("something");

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
