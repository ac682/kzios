use crate::syscall::put_char;
use core::fmt::{Arguments, Result, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        for i in s.chars() {
            put_char(i as usize);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}
