use core::fmt::Arguments;

use alloc::fmt::format;

use crate::call::sys_debug;

#[macro_export]
macro_rules! dbg
{
	($($arg:tt)*) => {{
		$crate::dbg::debug(format_args!($($arg)*));
    }};
}
pub fn debug(args: Arguments) {
    let str = format(args);
    unsafe { if let Err(_) = sys_debug(&str) {
        // nothing happens
    } }
}
