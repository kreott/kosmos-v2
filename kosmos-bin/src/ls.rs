#![no_std]
#![no_main]

extern crate alloc;

use kosmos_std::sys;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    // for now just print help to test the arg system
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    sys::write(1, b"panicked");
    loop {}
}