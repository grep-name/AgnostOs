#![no_main]
#![no_std]

use core::panic::PanicInfo;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    uefi::println!("Hello From RUST");

    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
