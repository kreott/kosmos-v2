//! sys.rs: contains wrappers for syscalls

pub fn read(fd: u64, buf: &[u8]) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0u64,
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
            lateout("rax") ret,
            options(nostack)
        );
    }
    ret
}

pub fn write(fd: u64, buf: &[u8]) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 1u64,
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
            lateout("rax") ret,
            options(nostack),
        );
    }
    ret
}

pub fn open(path: &str) -> u64 {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    // buf[len] is already 0 from initialization
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 2u64,
            in("rdi") buf.as_ptr(),
            in("rsi") 0u64,
            lateout("rax") ret,
            options(nostack),
        );
    }
    ret
}

pub fn exit(status: u64) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 60u64,
            in("rdi") status,
            options(nostack),
        );
    }
}