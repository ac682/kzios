use core::{
    arch::asm,
    fmt::{Arguments, Result, Write},
    panic::PanicInfo,
};

use crate::primitive::{qemu::UART, uart::Uart};

struct StdOut;

impl Write for StdOut {
    fn write_str(&mut self, s: &str) -> Result {
        for char in s.chars() {
            UART.write(char as u8);
        }
        Ok(())
    }
}

pub fn print(args: Arguments) {
    StdOut.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::lang_items::print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::lang_items::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

// #[no_mangle]
// extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(location) = info.location() {
        println!(
            "file {}, {}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}
