use crate::filesystem::file_descriptor::FD_TABLE;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::macros::*;


// process exit flag
pub static PROCESS_EXITED: AtomicBool = AtomicBool::new(false);

// kernel context for process exit
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct KernelContext {
    pub rsp: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
}

pub static KERNEL_CONTEXT: Mutex<KernelContext> = Mutex::new(KernelContext {
    rsp: 0, rbp: 0, rbx: 0, r12: 0, r13: 0, r14: 0, r15: 0, rip: 0,
});

// exit stack
pub static EXIT_STACK_TOP: AtomicU64 = AtomicU64::new(0);

pub fn init_exit_stack() {
    use crate::memory::{MAPPER, FRAME_ALLOCATOR};
    use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB, FrameAllocator, Mapper};

    const EXIT_STACK_VIRT: u64 = 0xFFFF_FF00_0000_0000;
    const STACK_PAGES: u64 = 4;

    let mut mapper = MAPPER.lock();
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let mapper = mapper.as_mut().unwrap();
    let frame_allocator = frame_allocator.as_mut().unwrap();

    for i in 0..STACK_PAGES {
        let page = Page::<Size4KiB>::containing_address(
            x86_64::VirtAddr::new(EXIT_STACK_VIRT + i * 4096)
        );
        let frame = frame_allocator.allocate_frame().expect("out of frames");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)
                .expect("exit stack map failed")
                .flush();
        }
    }

    EXIT_STACK_TOP.store(EXIT_STACK_VIRT + STACK_PAGES * 4096, Ordering::SeqCst);
}

#[unsafe(naked)]
pub unsafe extern "C" fn save_kernel_context(ctx: *mut KernelContext) {
    core::arch::naked_asm!(
        "mov [rdi],    rsp",
        "mov [rdi+8],  rbp",
        "mov [rdi+16], rbx",
        "mov [rdi+24], r12",
        "mov [rdi+32], r13",
        "mov [rdi+40], r14",
        "mov [rdi+48], r15",
        "mov rax, [rsp]",       // grab return address
        "mov [rdi+56], rax",
        "xor eax, eax",         // return 0
        "ret",
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn restore_kernel_context(ctx: *const KernelContext) -> ! {
    core::arch::naked_asm!(
        "mov rsp, [rdi]",
        "mov rbp, [rdi+8]",
        "mov rbx, [rdi+16]",
        "mov r12, [rdi+24]",
        "mov r13, [rdi+32]",
        "mov r14, [rdi+40]",
        "mov r15, [rdi+48]",
        "mov rax, [rdi+56]",    // return address
        "push rax",
        "xor eax, eax",
        "ret",                  // jump back to save_kernel_context call site
    );
}

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
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
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

    crate::filesystem::file_descriptor::open(path)
        .map(|fd| fd)
        .unwrap_or(u64::MAX)
}

fn sys_close(fd: u64) -> u64 {
    crate::filesystem::file_descriptor::close(fd) as u64
}

fn sys_exit(status: u64) -> u64 {
    PROCESS_EXITED.store(true, Ordering::SeqCst);
    serial_println!("process exited with status {}", status);

    let ctx = {
        let guard = KERNEL_CONTEXT.lock();
        *guard
    };

    let exit_stack = EXIT_STACK_TOP.load(Ordering::SeqCst);

    unsafe {
        core::arch::asm!(
            "mov rsp, {stack}",
            "mov rdi, {ctx}",
            "call {restore}",
            stack   = in(reg) exit_stack,
            ctx     = in(reg) &ctx as *const _ as u64,
            restore = sym restore_kernel_context,
            options(noreturn)
        );
    }
}

fn sys_getdents64(fd: u64, buf_ptr: u64, count: u64) -> u64 {
    let table = FD_TABLE.lock();
    let file = match table.get(&fd) {
        Some(f) if f.is_dir => f,
        _ => return u64::MAX,
    };
    let path = file.path.clone();
    drop(table);

    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count as usize) };
    let mut written = 0usize;

    crate::filesystem::fat32::with_fs(|fs| {
        let trimmed = path.trim_start_matches('/');
        
        // fix 1: root dir can't use open_dir(""), use root_dir() directly
        let dir = if trimmed.is_empty() {
            fs.root_dir()
        } else {
            match fs.root_dir().open_dir(trimmed) {
                Ok(d) => d,
                Err(_) => return,
            }
        };

        for entry in dir.iter() {
            let entry = match entry { Ok(e) => e, Err(_) => continue };
            let name = entry.file_name();
            let name_bytes = name.as_bytes();
            // use 19 as base because size_of Dirent64 is 19 + padding otherwise
            let reclen = (19 + name_bytes.len() + 1 + 7) & !7;

            if written + reclen > buf.len() { break; }

            unsafe {
                let base = buf.as_mut_ptr().add(written);
                let dirent = base as *mut Dirent64;
                (*dirent).d_ino = 1;
                (*dirent).d_off = (written + reclen) as i64;
                (*dirent).d_reclen = reclen as u16;
                (*dirent).d_type = if entry.is_dir() { 4 } else { 8 };
                let name_ptr = base.add(19);
                core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name_ptr, name_bytes.len());
                *name_ptr.add(name_bytes.len()) = 0;
            }
            written += reclen;
        }
    });

    written as u64
}