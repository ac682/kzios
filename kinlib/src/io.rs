use crate::syscall::sys_write;
use core::fmt::{Arguments, Result, Write};

/// Standard output stream
pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        for i in s.chars() {
            sys_write(i as usize);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    core::fmt::write(&mut Stdout, args).unwrap();
}
