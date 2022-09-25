use crate::process::Termination;
use core::arch::global_asm;

global_asm!(include_str!("rt.asm"));

#[export_name = "lang_start"]
#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    main().to_exit_code()
}