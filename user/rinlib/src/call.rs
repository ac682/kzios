use core::arch::asm;

use erhino_shared::{
    call::{SystemCall, SystemCallError},
    mem::Address,
    proc::{ExitCode, Tid},
};
use num_traits::FromPrimitive;

fn to_error(error: usize) -> SystemCallError {
    if let Some(ret) = SystemCallError::from_usize(error) {
        ret
    } else {
        SystemCallError::Unknown
    }
}

unsafe fn raw_call(id: usize, arg0: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    let mut error_code;
    let mut result;
    asm!("ecall", in("x17") id, inlateout("x10") arg0 => error_code, inlateout("x11") arg1 => result, in("x12") arg2);
    (error_code, result)
}

unsafe fn sys_call(
    call: SystemCall,
    arg0: usize,
    arg1: usize,
    arg2: usize,
) -> Result<usize, SystemCallError> {
    let (error, ret) = raw_call(call as usize, arg0, arg1, arg2);
    if error == 0 {
        Ok(ret)
    } else {
        Err(to_error(error))
    }
}

// returns actual byte count sent to debug stream
pub unsafe fn sys_debug(msg: &str) -> Result<usize, SystemCallError> {
    sys_call(SystemCall::Debug, msg.as_ptr() as usize, msg.len(), 0)
}

// returns the new heap top address, or the current when size is 0
pub unsafe fn sys_extend(size: usize) -> Result<Address, SystemCallError> {
    sys_call(SystemCall::Extend, size, 0, 0)
}

// returns nothing
pub unsafe fn sys_exit(code: ExitCode) -> Result<(), SystemCallError> {
    sys_call(SystemCall::Exit, code as usize, 0, 0).map(|_| ())
}

pub unsafe fn sys_thread_spawn(func_point: Address) -> Result<Tid, SystemCallError> {
    sys_call(SystemCall::ThreadSpawn, func_point, 0, 0).map(|t| t as Tid)
}

pub unsafe fn sys_tunnel_build() -> Result<usize, SystemCallError> {
    sys_call(SystemCall::TunnelBuild, 0, 0, 0)
}

pub unsafe fn sys_tunnel_link(key: usize) -> Result<Address, SystemCallError> {
    sys_call(SystemCall::TunnelLink, key, 0, 0)
}

pub unsafe fn sys_tunnel_dispose(key: usize) -> Result<(), SystemCallError>{
    sys_call(SystemCall::TunnelDispose, key, 0, 0).map(|_| {})
}