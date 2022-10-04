use core::{arch::asm, panic::PanicInfo, fmt::Arguments};
use erhino_shared::process::Termination;

use crate::{mm, print, println, trap};

#[lang = "start"]
fn rust_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    unsafe{ board_init(); }
    mm::init();
    trap::init();
    println!("boot stage #2: board initialization");
    main();
    println!("unreachable here");
    unsafe{
        asm!("ebreak");
        loop {
            asm!("wfi");
        }
    }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    print!("Kernel panicking: ");
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
    unsafe {
        loop {
            asm!("wfi");
        }
    }
}

extern "Rust"{
    #[linkage = "extern_weak"]
    pub fn board_init();
    #[linkage = "extern_weak"]
    pub fn write_out(args: Arguments);
}

#[macro_export]
macro_rules! print
{
	($($arg:tt)*) => {{
        unsafe {$crate::rt::write_out(format_args!($($arg)*));}
    }};
}

#[macro_export]
macro_rules! println
{
	() => ({
        use $crate::print;
		print!("\r\n")
	});
	($fmt:expr) => ({
        use $crate::print;
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
        use $crate::print;
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}
