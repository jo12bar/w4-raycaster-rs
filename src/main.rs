#![no_main]
#![no_std]

use core::{arch::wasm32, panic::PanicInfo};

const GAMEPAD1: *const u8 = 0x16 as *const u8;

const BUTTON_LEFT: u8 = 16; // 00010000
const BUTTON_RIGHT: u8 = 32; // 00100000
const BUTTON_UP: u8 = 64; // 01000000
const BUTTON_DOWN: u8 = 128; // 10000000

extern "C" {
    fn vline(x: i32, y: i32, len: u32);
}

#[panic_handler]
fn phandler(_: &PanicInfo<'_>) -> ! {
    wasm32::unreachable()
}

#[no_mangle]
unsafe fn update() {
    if *GAMEPAD1 & BUTTON_UP != 0 {
        vline(80, 20, 120);
    }
}
