#![no_std]
#![no_main]

use kosmos_std::prelude::*;
use kosmos_std::sys;

#[unsafe(no_mangle)]
pub extern "C" fn _start(argc: usize, argv: *const &str) -> ! {
    sys::write(1, b"ls: start\n");
    let path = if argc > 1 {
        let args = unsafe { core::slice::from_raw_parts(argv, argc) };
        args[1]
    } else {
        "/"
    };
    
    sys::write(1, b"ls: opening path\n");
    let fd = sys::open(path);
    sys::write(1, b"ls: got fd\n");
    
    if fd == u64::MAX {
        sys::write(2, b"ls: cannot open directory\n");
        sys::exit(1);
    }

    let mut buf = [0u8; 4096];
    sys::write(1, b"ls: calling getdents\n");
    let n = sys::getdents64(fd, &mut buf) as usize;
    sys::write(1, b"ls: got entries\n");

    let mut offset = 0;
    while offset < n {
        // reclen is at offset 18 (8 d_ino + 8 d_off + 2 d_reclen)
        let reclen = u16::from_le_bytes([buf[offset + 16], buf[offset + 17]]) as usize;
        // d_type is at offset 18
        // name starts at offset 19
        let name_start = offset + 19;
        let mut name_len = 0;
        while buf[name_start + name_len] != 0 { name_len += 1; }
        let name = core::str::from_utf8(&buf[name_start..name_start + name_len]).unwrap_or("?");
        sys::write(1, name.as_bytes());
        sys::write(1, b"\n");
        offset += reclen;
    }

    sys::exit(0);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    sys::write(1, info.message().as_str().unwrap().as_bytes());
    sys::write(1, b"panicked");
    sys::exit(1);
}