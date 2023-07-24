use core::fmt::Display;

use riscv::register::scause::{Exception, Interrupt, Scause, Trap};

use crate::{hart, println};

#[derive(Debug)]
pub enum TrapCause {
    Unknown,
    SoftwareInterrupt,
    ExternalInterrupt,
    TimerInterrupt,
    EnvironmentCall,
    Breakpoint,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    // 0-255
    pub x: [u64; 32],
    // 256 - 511
    pub f: [u64; 32],
    // 512-519
    pub satp: u64,
    // 520-527
    pub pc: u64,
    // 528
    /// Currently the hart it running in. Guaranteed by trap_vector in assembly.asm
    pub hartid: u64,
}

impl TrapFrame {
    pub const fn new(hartid: usize) -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0,
            pc: 0,
            hartid: hartid as u64,
        }
    }
}

impl Display for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Registers with hartid={}", self.hartid)?;
        writeln!(f, "ra={:#016x}, sp={:#016x}", self.x[1], self.x[2])?;
        writeln!(f, "gp={:#016x}, tp={:#016x}", self.x[3], self.x[4])?;
        writeln!(f, "fp={:#016x}", self.x[8])?;
        writeln!(f, "a0={:#016x}, a1={:#016x}", self.x[10], self.x[11])?;
        writeln!(f, "a2={:#016x}, a3={:#016x}", self.x[12], self.x[13])?;
        writeln!(f, "a4={:#016x}, a5={:#016x}", self.x[14], self.x[15])?;
        writeln!(f, "a6={:#016x}, a7={:#016x}", self.x[16], self.x[17])?;
        writeln!(f, "sepc={:#x}, satp={:#x}", self.pc, self.satp)
    }
}

#[no_mangle]
unsafe fn handle_trap(frame: &mut TrapFrame, cause: Scause, _val: usize) -> &TrapFrame {
    let hart = hart::this_hart();
    match cause.cause() {
        Trap::Interrupt(Interrupt::UserTimer) => hart.handle_user_trap(TrapCause::TimerInterrupt),
        Trap::Interrupt(Interrupt::SupervisorTimer) => todo!("nested interrupt: timer"),
        Trap::Interrupt(Interrupt::UserSoft) => todo!("impossible user soft interrupt"),
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            hart.clear_ipi();

            todo!("setup to enter userspace")
        }
        Trap::Exception(exception) => {
            frame.pc += 4;
            match exception {
                Exception::Breakpoint => hart.handle_user_trap(TrapCause::Breakpoint),
                Exception::UserEnvCall => hart.handle_user_trap(TrapCause::EnvironmentCall),
                _ => todo!("Unknown exception: {}", cause.bits()),
            }
        }
        _ => {
            todo!("Unknown trap: {}", cause.bits())
        }
    }
}
