use core::{arch::asm, ffi::CStr};

use erhino_shared::{
    call::SystemCall,
    mem::Address,
    proc::{ExitCode, Pid, ProcessInfo, Signal},
    service::Sid,
};

unsafe fn raw_call(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let mut _ret = 0usize;
    asm!("ecall", in("x17") id, inlateout("x10") arg0 => _ret, in("x11") arg1, in("x12") arg2, in("x13") arg3);
    _ret
}

pub unsafe fn sys_debug(msg: &CStr) {
    raw_call(SystemCall::Debug as usize, msg.as_ptr() as usize, 0, 0, 0);
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

pub unsafe fn sys_inspect(pid: Pid, info: &mut ProcessInfo, name_buffer: &mut [u8; 256]) -> bool {
    raw_call(
        SystemCall::Inspect as usize,
        info as *mut ProcessInfo as Address,
        pid as usize,
        name_buffer.as_ptr() as usize,
        0,
    ) == 0
}

pub unsafe fn sys_inspect_myself(info: &mut ProcessInfo, name_buffer: &mut [u8; 256]) -> bool {
    raw_call(
        SystemCall::InspectMyself as usize,
        info as *mut ProcessInfo as Address,
        0,
        name_buffer.as_ptr() as usize,
        0,
    ) == 0
}

pub unsafe fn sys_wait() -> bool {
    raw_call(SystemCall::Wait as usize, 0, 0, 0, 0) == 0
}

pub unsafe fn sys_wait_for(pid: Pid) -> ExitCode {
    raw_call(SystemCall::WaitFor as usize, pid as usize, 0, 0, 0) as ExitCode
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

// 返回值应该预先对 sys_call 有定义，大概就是负数表示错误，由于 sys_call 都是进程调用，所以先定义
// -1 为错误，但没有细节
// -7 为权限不足，先这样子
pub unsafe fn sys_service_register(sid: Sid) -> bool {
    raw_call(SystemCall::ServiceRegister as usize, sid, 0, 0, 0) == 0
}

// 通过 sys_signal_set(任意值, 0) 就可以在形式上彻底取消 handler
pub unsafe fn sys_signal_set(handler: Address, mask: Signal) {
    raw_call(SystemCall::SignalSet as usize, handler, mask as usize, 0, 0);
}

/// flags: 00000XWR
pub unsafe fn sys_extend(start: Address, count: usize, flags: u8) -> bool {
    raw_call(SystemCall::Extend as usize, start, count, flags as usize, 0) == 0
}
