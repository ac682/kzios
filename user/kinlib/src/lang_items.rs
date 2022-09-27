use core::panic::PanicInfo;

use crate::syscall::exit;

// #[no_mangle]
// extern "C" fn eh_personality() {}

/// Write data to stdout
#[macro_export]
macro_rules! print
{
	($($arg:tt)*) => {{
        $crate::io::_print(format_args!($($arg)*));
    }};
}

/// Write data to stdout with end line at the end
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
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
    exit(-1);
    loop{}
}
