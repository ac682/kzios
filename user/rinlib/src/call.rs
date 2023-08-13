use core::{arch::asm, ffi::CStr};

use erhino_shared::{
    call::{SystemCall, SystemCallError},
    mem::Address,
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

// returns actual byte count sent to debug stream
pub unsafe fn sys_debug(msg: &str) -> Result<usize, SystemCallError> {
    let (error, ret) = raw_call(
        SystemCall::Debug as usize,
        msg.as_ptr() as usize,
        msg.len(),
        0,
    );
    if error == 0 {
        Ok(ret)
    } else {
        Err(to_error(error))
    }
}

// returns the new heap top address, or the current when size is 0
pub unsafe fn sys_extend(size: usize) -> Result<Address, SystemCallError> {
    let (error, ret) = raw_call(SystemCall::Extend as usize, size, 0, 0);
    if error == 0 {
        Ok(ret as Address)
    } else {
        Err(to_error(error))
    }
}
