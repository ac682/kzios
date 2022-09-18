use crate::process::Termination;
use crate::syscall::exit;

#[lang = "start"]
fn _start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    main().to_exit_code()
}
