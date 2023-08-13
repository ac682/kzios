use core::fmt::Arguments;

use alloc::fmt::format;

use crate::call::sys_debug;

#[macro_export]
macro_rules! dbg
{
	($($arg:tt)*) => {{
		$crate::io::debug(format_args!($($arg)*));
    }};
}
pub fn debug(args: Arguments) {
    let str = format(args);
    unsafe {
        sys_debug(&str);
    }
}
