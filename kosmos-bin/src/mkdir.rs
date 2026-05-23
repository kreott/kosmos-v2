#![no_std]
#![no_main]

use kosmos_std::sys;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    sys::write(1, b"hello from mkdir\n");
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    sys::write(1, b"panicked");
    sys::exit(1);
    loop {}
}