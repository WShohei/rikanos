#![no_std]
pub mod ascii_font;
pub mod console;
pub mod font;
pub mod graphics;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

use core::fmt::Write;
pub fn _print(args: core::fmt::Arguments) {
    let console = console::Console::instance();
    console.write_fmt(args).unwrap();
}
