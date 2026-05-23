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


pub fn load_and_run(path: &str) -> bool {
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

        // make sure virtaddr is mapped
        map_segment(ph.virtual_addr(), ph.mem_size());

        unsafe {
            // copy segment data
            core::ptr::copy_nonoverlapping(
                data.as_ptr().add(offset),
                virtaddr,
                file_size,
                );
            // zero out the rest (bss)
            if mem_size > file_size {
                core::ptr::write_bytes(virtaddr.add(file_size), 0, mem_size - file_size);
            }
        }
        
    }

    // jump to elf entry point
    let entry: unsafe extern "C" fn() = unsafe {
        core::mem::transmute(elf.header.pt2.entry_point() as *const ())
    };
    PROCESS_EXITED.store(false, Ordering::SeqCst);
    unsafe { entry(); }
    true
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

fn map_segment(virtaddr: u64, size: u64) {
    let mut mapper = MAPPER.lock();
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let mapper = mapper.as_mut().unwrap();
    let frame_allocator = frame_allocator.as_mut().unwrap();

    let start = Page::<Size4KiB>::containing_address(VirtAddr::new(virtaddr));
    let end = Page::<Size4KiB>::containing_address(VirtAddr::new(virtaddr + size - 1));

    for page in Page::range_inclusive(start, end) {
        let frame = frame_allocator.allocate_frame().expect("out of frames");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            match mapper.map_to(page, frame, flags, frame_allocator) {
                Ok(flusher) => { flusher.flush() }
                Err(MapToError::PageAlreadyMapped(_)) => {
                    // update flags on existing mapping
                    mapper.update_flags(page, flags).unwrap().flush();
                }
                Err(e) => panic!("map_to failed: {:?}", e),
            }
        }
    }
}