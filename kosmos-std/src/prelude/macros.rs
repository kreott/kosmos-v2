use core::fmt;

struct SysWriter;

impl fmt::Write for SysWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        crate::sys::write(1, s.as_bytes());
        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SysWriter.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => ($crate::prelude::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_println {
    () => ($crate::prelude::macros::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
}