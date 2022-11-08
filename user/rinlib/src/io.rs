use core::fmt::{Arguments, Write};

use crate::call::sys_debug;

#[macro_export]
macro_rules! dbg
{
	($($arg:tt)*) => {{
		$crate::io::debug(format_args!($($arg)*));
    }};
}

pub fn debug(args: Arguments) {
    KernelDebugOutput.write_fmt(args).unwrap();
}

pub struct KernelDebugOutput;

impl Write for KernelDebugOutput {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { sys_debug(s) };
        Ok(())
    }
}
