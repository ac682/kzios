use core::fmt::{Arguments, Error, Result, Write};

use erhino_shared::sync::DataLock;

use crate::{
    board,
    sbi::{self, SbiExtension},
    sync::hart::HartLock,
};

static mut LOCKED_CONSOLE: DataLock<Console, HartLock> = DataLock::new(Console, HartLock::new());

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
        $crate::print!(concat!("\x1b[0;35mDEBG\x1b[0m ", $fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
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
    fn write_str(&mut self, s: &str) -> Result {
        if board::is_board_ready()
            && board::this_board()
                .see()
                .is_extension_supported(SbiExtension::DebugConsole)
        {
            match sbi::debug_console_write(s) {
                Ok(_res) => Ok(()),
                Err(_err) => Err(Error::default()),
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
    unsafe {
        let mut console = LOCKED_CONSOLE.lock();
        console.write_fmt(args).unwrap();
    }
}
