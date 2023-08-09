use core::fmt::Display;

use erhino_shared::call::SystemCall;
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
    // 以下是临时数据，与 TrapFrame 所属的进程无关
    // 528 Currently the hart it running in. Guaranteed by trap_vector in assembly.asm
    hartid: u64,

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

pub struct TrapContext{
    // process reference
    // thread reference
}

pub struct EnvironmentCallBody{
    pub function: SystemCall,

}

#[no_mangle]
unsafe fn handle_trap(frame: &mut TrapFrame, cause: Scause, _val: usize) -> &TrapFrame {
    // NOTE: frame 指针有可能是 0，但这种情况只出现在每个 hart 的第一次 trap 中，也就是 SupervisorSoft 中，故不必检查 frame 有效性（但这么做挺冒险
    let hart = hart::this_hart();
    match cause.cause() {
        Trap::Interrupt(Interrupt::UserTimer) => hart.trap(TrapCause::TimerInterrupt),
        Trap::Interrupt(Interrupt::SupervisorTimer) => todo!("nested interrupt: timer"),
        Trap::Interrupt(Interrupt::UserSoft) => todo!("impossible user soft interrupt"),
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            hart.clear_ipi();
            hart.trap(TrapCause::SoftwareInterrupt);
        }
        Trap::Exception(exception) => {
            frame.pc += 4;
            match exception {
                Exception::Breakpoint => hart.trap(TrapCause::Breakpoint),
                Exception::UserEnvCall => hart.trap(TrapCause::EnvironmentCall),
                _ => todo!("Unknown exception: {}", cause.bits()),
            }
        }
        _ => {
            todo!("Unknown trap: {}", cause.bits())
        }
    }
    hart.arranged_frame()
}
