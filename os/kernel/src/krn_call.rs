use core::arch::asm;

use erhino_shared::call::KernelCall;

fn raw_cal(call: KernelCall, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let mut ret = 0usize;
    unsafe {
        asm!("ecall", in("x17") call as usize, inlateout("x10") arg0 => ret, in("x11") arg1, in("x12") arg2, in("x13") arg2);
    }
    ret
}

pub fn krn_enter_user_space() {
    raw_cal(KernelCall::EnterUserSpace, 0, 0, 0, 0);
}
