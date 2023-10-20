use core::fmt::{Arguments, Write};

use erhino_shared::sync::spin::SimpleLock;
use lock_api::Mutex;

use crate::sbi::{self};

static LOCKED_CONSOLE: Mutex<SimpleLock, Console> = Mutex::new(Console);

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
		$crate::print!("\n")
	});
	($fmt:expr) => ({
		$crate::print!(concat!($fmt, "\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		$crate::print!(concat!($fmt, "\n"), $($args)+)
	});
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr) => ({
        #[cfg(debug_assertions)]
        $crate::print!(concat!("\x1b[0;35mDEBG\x1b[0m ", $fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        #[cfg(debug_assertions)]
        $crate::print!(concat!("\x1b[0;35mDEBG\x1b[0m ", $fmt, "\n"), $($args)+)
    });
}

#[macro_export]
macro_rules! info {
    ($fmt:expr) => ({
        $crate::print!(concat!("\x1b[0;32mINFO\x1b[0m ", $fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!("\x1b[0;32mINFO\x1b[0m ", $fmt, "\n"), $($args)+)
    });
}

#[macro_export]
macro_rules! warning {
    ($fmt:expr) => ({
        $crate::print!(concat!("\x1b[0;33mWARN\x1b[0m ", $fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!("\x1b[0;33mWARN\x1b[0m ", $fmt, "\n"), $($args)+)
    });
}

#[macro_export]
macro_rules! error {
    ($fmt:expr) => ({
        $crate::print!(concat!("\x1b[0;31mERRO\x1b[0m ", $fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!("\x1b[0;31mERRO\x1b[0m ", $fmt, "\n"), $($args)+)
    });
}

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if sbi::is_debug_console_supported() {
            match sbi::debug_console_write(s) {
                Ok(_res) => Ok(()),
                Err(_err) => Err(core::fmt::Error::default()),
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
    // SpinLock is causing deadlock while trap occurred
    // However HartLock is too expensive
    let mut console = LOCKED_CONSOLE.lock();
    console.write_fmt(args).unwrap();
}
