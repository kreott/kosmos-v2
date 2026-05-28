#![no_std]
#![no_main]

use kosmos_std::prelude::*;
use kosmos_std::sys;
use kosmos_std::args::{Args, Help};

#[unsafe(no_mangle)]
pub extern "C" fn _start(argc: usize, argv: *const &str) -> ! {
    let argv = kosmos_std::init_args(argc, argv);
    let args = Args::parse(argv);

    if args.flag("h") || args.flag("help") {
        Help::new("ls", "list directory contents")
            .flag("a", "all",  "show hidden files")
            .flag("l", "long", "long listing format")
            .print();
        sys::exit(0);
    }
    
    let show_all = args.flag("a") || args.flag("all");
    let path = args.positional(0, "/");

    let fd = sys::open(path.as_str());    
    if fd == u64::MAX {
        sys::write(2, b"ls: could not open directory\n");
        sys::exit(1);
    }

    let mut buf = [0u8; 4096];
    let n = sys::getdents64(fd, &mut buf) as usize;

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

        if !show_all && name.starts_with(".") {
            offset += reclen;
            continue;
        }

        vga_print!("{}  ", name);
        offset += reclen;
    }

    vga_print!("\n");
    sys::exit(0);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    sys::write(1, info.message().as_str().unwrap().as_bytes());
    sys::write(1, b"panicked");
    sys::exit(1);
}