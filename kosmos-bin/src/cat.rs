#![no_std]
#![no_main]

use kosmos_std::sys;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    sys::write(1, b"hello from cat\n");
    sys::exit(0);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    sys::write(1, info.message().as_str().unwrap().as_bytes());
    sys::write(1, b"panicked");
    sys::exit(1);
}