#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kosmos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use kosmos::task::shell::shell_task;
use core::panic::PanicInfo;
use kosmos::macros::*;
use kosmos::task::Task;
use kosmos::{allocator, task::executor::Executor};
use kosmos::syscall;

entry_point!(kernel_main);

#[allow(unreachable_code)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use kosmos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    // initializations
    clear!();
    println!("Hello World{}", "!");
    kosmos::init();

    // initialize memory and heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    memory::init_globals(mapper, frame_allocator);
    syscall::init_exit_stack();


    #[cfg(test)]
    {
        test_main();
        kosmos::exit_qemu(kosmos::QemuExitCode::Success);
    }

    // async task executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(shell_task()));
    executor.run();

    // executor.run() never returns so the loop is a fallback just in case
    kosmos::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    serial_println!("{}", info);
    kosmos::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kosmos::test_panic_handler(info)
}