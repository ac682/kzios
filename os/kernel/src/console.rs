use core::fmt::Arguments;

extern "Rust"{
    #[linkage = "extern_weak"]
    pub fn write_out(args: Arguments);
}

#[macro_export]
macro_rules! print
{
	($($arg:tt)*) => {{
        unsafe {$crate::console::write_out(format_args!($($arg)*));}
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