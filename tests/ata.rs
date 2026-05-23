#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kosmos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use kosmos::filesystem::fat32::Disk;
use kosmos::drivers::ata::{AtaDrive, AtaBus, AtaUnit};
use kosmos::macros::*;
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use kosmos::allocator;
    use kosmos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    kosmos::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kosmos::test_panic_handler(info)
}

#[test_case]
pub fn test_ata_fat32() {
    serial_println!("test start");
    let mut drive = AtaDrive::new_with(AtaBus::Secondary, AtaUnit::Master);
    serial_println!("detected: {}", drive.detect());
    let disk = Disk::new(drive);
    let fs = fatfs::FileSystem::new(disk, fatfs::FsOptions::new()).unwrap();
    let root = fs.root_dir();
    for entry in root.iter() {
        serial_println!("{}", entry.unwrap().file_name());
    }
}