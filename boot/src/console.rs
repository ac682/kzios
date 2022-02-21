use core::fmt::{Arguments, Result, Write};

use crate::sbi::console_putchar;

struct Stdout;

impl Write for Stdout
{
    fn write_str(&mut self, s: &str) -> Result
    {
        for c in s.chars()
        {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: Arguments)
{
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}