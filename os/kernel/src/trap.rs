use core::fmt::Display;
use riscv::register::mcause::Mcause;

use crate::hart::of_hart;

#[no_mangle]
unsafe fn handle_trap(hartid: usize, cause: Mcause, val: usize) -> &'static TrapFrame {
    // 这里要区分 trap，from machine 和 from user 区别对待
    let hart = of_hart(hartid);
    hart.handle_trap_from_user(cause, val);
    hart.context()
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TrapFrame {
    // 0-255
    pub x: [u64; 32],
    // 256 - 511
    pub f: [u64; 32],
    // 512-519
    pub satp: u64,
    // 520-527
    pub pc: u64,
}

impl TrapFrame {
    pub const fn new() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0,
            pc: 0,
        }
    }
}

impl Display for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Registers")?;
        writeln!(f, "ra={:#016x}, sp={:#016x}", self.x[1], self.x[2])?;
        writeln!(f, "gp={:#016x}, tp={:#016x}", self.x[3], self.x[4])?;
        writeln!(f, "fp={:#016x}", self.x[8])?;
        writeln!(f, "a0={:#016x}, a1={:#016x}", self.x[10], self.x[11])?;
        writeln!(f, "a2={:#016x}, a3={:#016x}", self.x[12], self.x[13])?;
        writeln!(f, "a4={:#016x}, a5={:#016x}", self.x[14], self.x[15])?;
        writeln!(f, "a6={:#016x}, a7={:#016x}", self.x[16], self.x[17])?;
        writeln!(f, "mepc={:#x}, satp={:#x}", self.pc, self.satp)
    }
}
