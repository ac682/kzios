use core::arch::asm;

use crate::process::Pid;

fn raw_call(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut _ret = 0u64;
    unsafe {
        asm!("ecall", in("x17") id, inlateout("x10") arg0 => _ret, in("x11") arg1, in("x12") arg2, in("x13") arg3);
    }
    _ret
}

/// Write char to stdout
pub fn put_char(char: usize) {
    raw_call(0x0, char as u64, 0, 0, 0);
}

/// Process exit
pub fn exit(exit_code: i64) {
    raw_call(0x20, exit_code as u64, 0, 0, 0);
}

/// Fork current process and get its pid for the parent while zero for itself
///
/// + Pid<u32> Child itself
/// 0 Parent itself
/// - Errno:
/// -1 -> udf
/// -2 -> udf
pub fn fork() -> Result<Pid, ()> {
    let res = raw_call(0x21, 0, 0, 0, 0) as i64;
    if res < 0 {
        Err(())
    } else {
        Ok(res as Pid)
    }
}

/// Set current process's signal handler function entry point
pub fn signal_set(handler: fn(u8)) {
    let address = handler as usize;
    raw_call(0x31, address as u64, 0, 0, 0);
}
