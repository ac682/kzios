use core::arch::asm;

use erhino_shared::{call::SystemCall, mem::Address, process::ExitCode};

unsafe fn raw_call(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let mut _ret = 0usize;
    asm!("ecall", in("x17") id, inlateout("x10") arg0 => _ret, in("x11") arg1, in("x12") arg2, in("x13") arg3);
    _ret
}

pub unsafe fn sys_exit(code: ExitCode) {
    raw_call(SystemCall::Exit as usize, code as usize, 0, 0, 0);
}

pub unsafe fn sys_yield() {
    raw_call(SystemCall::Yield as usize, 0, 0, 0, 0);
}

/// flags: 00000XWR
pub unsafe fn sys_extend(start: Address, count: usize, flags: u8) {
    raw_call(SystemCall::Extend as usize, start, count, flags as usize, 0);
}
