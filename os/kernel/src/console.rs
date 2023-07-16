use core::fmt::{Arguments, Error, Result, Write};

use crate::sbi;

#[macro_export]
macro_rules! print
{
	($($arg:tt)*) => {{
		$crate::console::console_write(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println
{
	() => ({
        use $crate::print;
		print!("\r\n")
	});
	($fmt:expr) => ({
        use $crate::print;
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
        use $crate::print;
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {
        if sbi::is_debug_console_supported() {
            match sbi::debug_console_write(s) {
                Ok(res) => Ok(()),
                Err(err) => Err(Error::default()),
            }
        } else {
            for i in s.chars() {
                sbi::legacy_console_putchar(i as u8);
            }

            Ok(())
        }
    }
}

pub fn console_write(args: Arguments) {
    Console.write_fmt(args).unwrap();
}
