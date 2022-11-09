use core::{
    arch::asm,
    fmt::{Arguments, Write},
};

use alloc::{ffi::CString, fmt::format};

use crate::call::{sys_debug};

#[macro_export]
macro_rules! dbg
{
	($($arg:tt)*) => {{
		$crate::io::debug(format_args!($($arg)*));
    }};
}
pub fn debug(args: Arguments) {
    let str = format(args);
    let cstr = CString::new(str).unwrap();
    unsafe { sys_debug(&cstr) };
}
