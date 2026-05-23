use crate::macros::*;

pub fn dispatch(
    number: u64, 
    arg1: u64, 
    arg2: u64, 
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    match number {
        0  => sys_read(arg1, arg2, arg3),
        1  => sys_write(arg1, arg2, arg3),
        2  => sys_open(arg1, arg2),
        3  => sys_close(arg1),
        60 => sys_exit(arg1),
        _  => u64::MAX, // enosys
    }
}

fn sys_read(fd: u64, buf_ptr: u64, count: u64) -> u64 {
    let buf = unsafe {
        core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count as usize)
    };
    crate::filesystem::file_descriptor::read(fd, buf) as u64
}


fn sys_write(fd: u64, buf: u64, count: u64) -> u64 {
    if fd == 1 || fd == 2 {
        let slice = unsafe {
            core::slice::from_raw_parts(buf as *const u8, count as usize)
        };
        if let Ok(s) = core::str::from_utf8(slice) {
            print!("{}", s);
        }
        count
    } else {
        u64::MAX
    }
}

fn sys_open(path_ptr: u64, _flags: u64) -> u64 {
    let path = unsafe {
        let ptr = path_ptr as *const u8;
        let mut len = 0;
        while *ptr.add(len) != 0 { len += 1; }
        core::str::from_utf8(core::slice::from_raw_parts(ptr, len)).unwrap()
    };
    crate::filesystem::file_descriptor::open(path)
        .map(|fd| fd)
        .unwrap_or(u64::MAX)
}

fn sys_close(fd: u64) -> u64 {
    crate::filesystem::file_descriptor::close(fd) as u64
}

// temporary way to return to shell
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
pub static PROCESS_EXITED: AtomicBool = AtomicBool::new(false);

fn sys_exit(_status: u64) -> u64 {
    PROCESS_EXITED.store(true, Ordering::SeqCst);
    0
}