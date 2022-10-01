use core::arch::asm;

use crate::types::{PageNumber, Pid};

fn raw_call(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let mut _ret = 0usize;
    unsafe {
        asm!("ecall", in("x17") id, inlateout("x10") arg0 => _ret, in("x11") arg1, in("x12") arg2, in("x13") arg3);
    }
    _ret
}

/// Write char to stdout
pub fn sys_write(char: usize) {
    raw_call(0x0, char, 0, 0, 0);
}

/// Process exit
pub fn sys_exit(exit_code: i64) {
    raw_call(0x20, exit_code as usize, 0, 0, 0);
}

/// Fork current process and get its pid for the parent while zero for itself
///
/// + Pid<u32> Child itself
/// 0 Parent itself
/// - Errno:
/// -1 -> udf
/// -2 -> udf
pub fn sys_fork() -> Result<Pid, ()> {
    let res = raw_call(0x21, 0, 0, 0, 0) as i64;
    if res < 0 {
        Err(())
    } else {
        Ok(res as Pid)
    }
}

/// Set current process's signal handler function entry point
pub fn sys_signal_set(handler: fn(u8)) {
    let address = handler as usize;
    raw_call(0x31, address, 0, 0, 0);
}

/// Map vpn to somewhere from the kernel memory pool
/// flags:
/// Readable = 0b10
/// Writeable = 0b100
/// Executable = 0b1000
///
/// Its valid to set other bits to 1, but not safe and recommended
pub fn sys_map(vpn: PageNumber, count: usize, flags: usize) {
    raw_call(0x50, vpn as usize, count, flags, 0);
}
