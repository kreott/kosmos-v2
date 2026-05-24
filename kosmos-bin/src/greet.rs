#![no_std]
#![no_main]

use kosmos_std::sys;

#[unsafe(no_mangle)]
pub extern "C" fn _start(argc: usize, argv: *const &str) {
    let mut bytes = [0u8; 20];

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };
    let name = if argc > 1 { args[1] } else { "world" };
    let ret1 = sys::write(1, b"hello, ");
    sys::write(1, name.as_bytes());

    let hello = u64_to_buf(ret1, &mut bytes);

    sys::write(1, b"!\n");
    sys::write(1, hello);
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    sys::write(1, b"panicked");
    sys::exit(1);
    loop {}
}

fn u64_to_buf(mut n: u64, buf: &mut [u8; 20]) -> &[u8] {
    let mut i = 20;
    if n == 0 {
        i -= 1;
        buf[i] = b'0';
    }
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    &buf[i..]
}