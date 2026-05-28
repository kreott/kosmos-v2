use core::fmt;

struct SysWriter;
struct ErrSysWriter;

impl fmt::Write for SysWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        crate::sys::write(1, s.as_bytes());
        Ok(())
    }
}

impl fmt::Write for ErrSysWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        crate::sys::write(2, s.as_bytes());
        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SysWriter.write_fmt(args).unwrap();
}

pub fn _errprint(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SysWriter.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => ($crate::prelude::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_eprint {
    ($($arg:tt)*) => ($crate::prelude::macros::_errprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_println {
    () => ($crate::prelude::macros::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_eprintln {
    () => ($crate::prelude::macros::vga_eprint!("\n"));
    ($($arg:tt)*) => ($crate::vga_eprint!("{}\n", format_args!($($arg)*)));
}