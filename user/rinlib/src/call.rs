use core::arch::asm;

use erhino_shared::{
    call::SystemCall,
    mem::Address,
    proc::{ExitCode, Pid, Signal, SystemSignal},
};

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

// perm: 0b0000_N.M.P.V.
pub unsafe fn sys_fork(perm: u8) -> i64 {
    raw_call(SystemCall::Fork as usize, perm as usize, 0, 0, 0) as i64
}

pub unsafe fn sys_signal_return() {
    raw_call(SystemCall::SignalReturn as usize, 0, 0, 0, 0);
}

pub unsafe fn sys_signal_send(pid: Pid, signal: Signal) {
    raw_call(
        SystemCall::SignalSend as usize,
        pid as usize,
        signal as usize,
        0,
        0,
    );
}

// 通过 sys_signal_set(任意值, 0) 就可以在形式上彻底取消 handler
pub unsafe fn sys_signal_set(handler: Address, mask: Signal) {
    raw_call(SystemCall::SignalSet as usize, handler, mask as usize, 0, 0);
}

/// flags: 00000XWR
pub unsafe fn sys_extend(start: Address, count: usize, flags: u8) -> bool {
    raw_call(SystemCall::Extend as usize, start, count, flags as usize, 0) == 0
}
