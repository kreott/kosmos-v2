use crate::filesystem::file_descriptor::FD_TABLE;
use crate::macros::*;

#[repr(C)]
struct Dirent64 {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
    // d_name follows immediately after, null terminated
}

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
        217 => sys_getdents64(arg1, arg2, arg3),
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

        serial_println!("sys_open: path={}", path);

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

fn sys_getdents64(fd: u64, buf_ptr: u64, count: u64) -> u64 {
    serial_println!("sys_getdents64 called! fd={}", fd);
    let table = FD_TABLE.lock();
    let file = match table.get(&fd) {
        Some(f) if f.is_dir => { serial_println!("found fd is_dir={}", f.is_dir); f },
        _ => { serial_println!("fd not found"); return u64::MAX; }
    };
    let path = file.path.clone();
    drop(table);

    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count as usize) };
    let mut written = 0usize;

    crate::filesystem::fat32::with_fs(|fs| {
        let dir = match fs.root_dir().open_dir(path.trim_start_matches('/')) {
            Ok(d) => d,
            Err(_) => return,
        };

        for entry in dir.iter() {
            let entry = match entry { Ok(e) => e, Err(_) => continue };
            let name = entry.file_name();
            let name_bytes = name.as_bytes();
            let reclen = (core::mem::size_of::<Dirent64>() + name_bytes.len() + 1 + 7) & !7;

            if written + reclen > buf.len() { break; }

            unsafe {
                let dirent = buf.as_mut_ptr().add(written) as *mut Dirent64;

                (*dirent).d_ino = 1; // fatfs has no idones, use 1
                (*dirent).d_off = (written + reclen) as i64;
                (*dirent).d_reclen = reclen as u16;
                (*dirent).d_type = if entry.is_dir() { 4 } else { 8 }; // DT_DIR=4, DT_REG=8
                let name_ptr = dirent.add(1) as *mut u8;
                core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name_ptr, name_bytes.len());
                *name_ptr.add(name_bytes.len()) = 0; // null terminator
            }
            written += reclen;
        }
    });


    serial_println!("getdents64: write {} bytes", written);
    serial_println!("getdents64: first bytes: {:x} {:x} {:x} {:x} {:x}", buf[0], buf[1], buf[2], buf[3], buf[4]);

    written as u64
}