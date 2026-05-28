use alloc::vec::Vec;
use xmas_elf::{ElfFile, program::Type};
use crate::macros::*;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError},
};
use crate::memory::{MAPPER, FRAME_ALLOCATOR};
use core::sync::atomic::Ordering;
use crate::syscall::PROCESS_EXITED;
use crate::syscall::KERNEL_CONTEXT;

pub fn load_and_run(path: &str, args: &[&str]) -> bool {
    serial_println!("loader: reading {}", path);
    let data = match read_file(path) {
        Some(d) => d,
        None => {
            return false;
        }
    };

    // verify it's an elf
    let elf = match ElfFile::new(data.as_slice()) {
        Ok(e) => e,
        Err(e) => {
            serial_println!("loader: invalid elf: {}", e);
            return false;
        }
    };

    // load PT_LOAD segments into memory
    for ph in elf.program_iter() {
        if ph.get_type().unwrap() != Type::Load { continue; }
        let virtaddr = ph.virtual_addr() as *mut u8;
        let offset = ph.offset() as usize;
        let file_size = ph.file_size() as usize;
        let mem_size = ph.mem_size() as usize;

        map_segment(ph.virtual_addr(), ph.mem_size());

        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr().add(offset),
                virtaddr,
                file_size,
            );
            if mem_size > file_size {
                core::ptr::write_bytes(virtaddr.add(file_size), 0, mem_size - file_size);
            }
        }
    }

    let entry_point = elf.header.pt2.entry_point();
    let stack_top = allocate_user_stack();
    let (argc, argv, new_sp) = push_args_to_stack(stack_top, args);

    // store entry info for potential re-entry detection
    PROCESS_EXITED.store(false, Ordering::SeqCst);

    // save rsp and rip via inline asm directly in this frame
    let mut saved_rsp: u64;
    let mut saved_rip: u64 = 0;
    
    unsafe {
        core::arch::asm!(
            "lea {rip}, [rip + 0]",  // get current rip
            rip = out(reg) saved_rip,
        );
        core::arch::asm!(
            "mov {rsp}, rsp",
            rsp = out(reg) saved_rsp,
        );
    }

    // if we got here via sys_exit restoring rsp, exit is true
    if PROCESS_EXITED.load(Ordering::SeqCst) {
        PROCESS_EXITED.store(false, Ordering::SeqCst);
        return true;
    }

    // store for sys_exit
    {
        let mut ctx = KERNEL_CONTEXT.lock();
        ctx.rsp = saved_rsp;
        ctx.rip = saved_rip;
    }

    jump_to_userspace(entry_point, new_sp, argc, argv);
}

fn read_file(path: &str) -> Option<Vec<u8>> {
    let mut data = Vec::new();
    let mut found = true;
    crate::filesystem::fat32::with_fs(|fs| {
        let file = fs.root_dir().open_file(path.trim_start_matches('/'));
        match file {
            Err(_) => { found = false; }
            Ok(mut f) => {
                let mut buf = [0u8; 512];
                loop {
                    let n = fatfs::Read::read(&mut f, &mut buf).unwrap();
                    if n == 0 { break; }
                    data.extend_from_slice(&buf[..n]);
                }
            }
        }
    });
    if found { Some(data) } else { None }
}

fn jump_to_userspace(entry: u64, stack_top: u64, argc: u64, argv: u64) -> ! {
    let user_cs = crate::gdt::GDT.1.user_code_selector.0 as u64;
    let user_ss = crate::gdt::GDT.1.user_data_selector.0 as u64;

    unsafe {
        core::arch::asm!(
            "mov ax, {ss:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "push {ss}",
            "push {rsp}",
            "push 0x200",
            "push {cs}",
            "push {rip}",
            "mov rdi, {argc}",
            "mov rsi, {argv}",
            "iretq",
            ss   = in(reg) user_ss,
            rsp  = in(reg) stack_top,
            cs   = in(reg) user_cs,
            rip  = in(reg) entry,
            argc = in(reg) argc,
            argv = in(reg) argv,
            options(noreturn)
        );
    }
}

fn map_segment(virtaddr: u64, size: u64) {
    let mut mapper = MAPPER.lock();
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let mapper = mapper.as_mut().unwrap();
    let frame_allocator = frame_allocator.as_mut().unwrap();

    let start = Page::<Size4KiB>::containing_address(VirtAddr::new(virtaddr));
    let end = Page::<Size4KiB>::containing_address(VirtAddr::new(virtaddr + size - 1));

    for page in Page::range_inclusive(start, end) {
        let frame = frame_allocator.allocate_frame().expect("out of frames");
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE;

        unsafe {
            match mapper.map_to(page, frame, flags, frame_allocator) {
                Ok(flusher) => { flusher.flush() }
                Err(MapToError::PageAlreadyMapped(_)) => {
                    mapper.update_flags(page, flags).unwrap().flush();
                }
                Err(e) => panic!("map_to failed: {:?}", e),
            }
        }
    }
}

fn allocate_user_stack() -> VirtAddr {
    const STACK_SIZE: usize = 4096 * 32;
    const STACK_TOP: u64 = 0x0000_7FFF_FF00_0000;

    let mut mapper = MAPPER.lock();
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let mapper = mapper.as_mut().unwrap();
    let frame_allocator = frame_allocator.as_mut().unwrap();

    let stack_start = VirtAddr::new(STACK_TOP - STACK_SIZE as u64);
    let stack_end = VirtAddr::new(STACK_TOP);

    let start_page = Page::<Size4KiB>::containing_address(stack_start);
    let end_page = Page::<Size4KiB>::containing_address(stack_end - 1u64);

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator.allocate_frame().expect("out of frames");
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            match mapper.map_to(page, frame, flags, frame_allocator) {
                Ok(flusher) => { flusher.flush(); }
                Err(MapToError::PageAlreadyMapped(_)) => {
                    mapper.update_flags(page, flags).unwrap().flush();
                }
                Err(e) => panic!("stack map failed: {:?}", e),
            }
        }
    }

    VirtAddr::new(STACK_TOP - 16)
}

fn push_args_to_stack(stack_top: VirtAddr, args: &[&str]) -> (u64, u64, u64) {
    let mut sp = stack_top.as_u64();

    // 1. write string bytes onto stack, record (ptr, len) for each
    let mut str_ptrs: Vec<(u64, u64)> = Vec::new();
    for arg in args.iter() {
        let bytes = arg.as_bytes();
        sp -= bytes.len() as u64;
        sp &= !0xF;
        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                sp as *mut u8,
                bytes.len(),
            );
        }
        str_ptrs.push((sp, bytes.len() as u64));
    }

    // 2. write &str fat pointers (ptr + len) onto stack
    sp -= (args.len() * 16) as u64;
    sp &= !0xF;
    let argv_ptr = sp;
    for (i, (ptr, len)) in str_ptrs.iter().enumerate() {
        unsafe {
            let slot = (sp + i as u64 * 16) as *mut u64;
            slot.write(*ptr);
            slot.add(1).write(*len);
        }
    }

    // 3. final alignment — rsp must be 16n-8 at _start entry
    sp -= 8;

    (args.len() as u64, argv_ptr, sp)
}