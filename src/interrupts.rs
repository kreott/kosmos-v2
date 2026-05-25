use crate::hlt_loop;
use crate::gdt;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::macros::*;
use crate::gdt::GDT;
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask};
use x86_64::registers::rflags::RFlags;



// IDT, InterruptDescriptorTable
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        unsafe {
            idt[InterruptIndex::Syscall.as_usize()].set_handler_addr(x86_64::VirtAddr::new(syscall_interrupt_handler as *const () as u64));
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Syscall = 0x80,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}


pub fn init_syscall() {
    unsafe {
        Efer::update(|f| *f |= EferFlags::SYSTEM_CALL_EXTENSIONS);

        LStar::write(x86_64::VirtAddr::new(
            syscall_interrupt_handler as *const () as u64,
        ));

        Star::write(
            GDT.1.user_code_selector,
            GDT.1.user_data_selector,
            GDT.1.code_selector,
            GDT.1.data_selector,
        ).unwrap();

        // Mask interrupts + trap flag during syscall handler
        // Without this, an IRQ can fire before you've saved rcx/r11
        SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::TRAP_FLAG);
    }
}

// handlers
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\nerror code: {}\n{:#?}", error_code, stack_frame);
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_interrupt_handler() {
    core::arch::naked_asm!(
        // rax = syscall number, rdi/rsi/rdx/r10/r8/r9 = args
        "push rcx",     // save return address (rcx clobbered by syscall)
        "push r11",     // save rflags     (r11 clobbered by syscall)
        // callee-saved registers
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Fix up args to match extern "C" calling convention:
        // dispatch(number, arg1, arg2, arg3, arg4, arg5)
        //          rdi     rsi   rdx   rcx   r8    r9
        // Currently: rax=number, rdi=arg1, rsi=arg2, rdx=arg3, r10=arg4, r8=arg5
        "mov r9,  r8",   // arg5: r8  → r9
        "mov r8,  r10",  // arg4: r10 → r8
        "mov rcx, rdx",  // arg3: rdx → rcx
        "mov rdx, rsi",  // arg2: rsi → rdx
        "mov rsi, rdi",  // arg1: rdi → rsi
        "mov rdi, rax",  // number: rax → rdi  ← this was missing!

        "call {dispatch}",

        // rax now holds return value — preserve it across pops
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "pop r11",      // restore rflags
        "pop rcx",      // restore return address
        "sysretq",
        dispatch = sym dispatch_from_asm,
    );
}

extern "C" fn dispatch_from_asm(number: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    crate::syscall::dispatch(number, arg1, arg2, arg3, arg4, arg5, 0)
}

// tests

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
