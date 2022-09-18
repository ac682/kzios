use core::arch::asm;

fn raw_call(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) {
    unsafe {
        asm!("ecall", in("x17") id, in("x10") arg0, in("x11") arg1, in("x12") arg2, in("x13") arg3);
    }
}

/// Write char to stdout
pub fn put_char(char: usize) {
    raw_call(0x0, char as u64, 0, 0, 0);
}

/// Process exit
pub fn exit(exit_code: i64) {
    raw_call(0x22, exit_code as u64, 0, 0, 0);
}
