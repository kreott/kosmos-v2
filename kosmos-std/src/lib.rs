#![no_std]

pub mod sys;
pub mod alloc;
pub mod args;
pub mod prelude;

pub fn init_args<'a>(argc: usize, argv: *const &'a str) -> &'a [&'a str] {
    if argc == 0 || argv.is_null() {
        &[]
    } else {
        unsafe { core::slice::from_raw_parts(argv, argc) }
    }
}