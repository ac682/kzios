use crate::syscall::put_char;
use core::fmt::Arguments;

#[doc(hidden)]
pub fn _print(args: Arguments) {
    for c in args.as_str().unwrap().chars() {
        put_char(c as usize);
    }
}
