use core::fmt::Display;

use erhino_shared::{call::SystemCall, mem::Address};
use riscv::register::scause::{Exception, Interrupt, Scause, Trap};

use crate::{
    external::{_kernel_end, _stack_size},
    hart,
    mm::unit::KERNEL_SATP,
    println,
};

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
    // 512
    pub pc: u64,
    // 以下是临时数据，与 TrapFrame 所属的进程无关
    // 520 Currently the hart it running in. Guaranteed by trap_vector in assembly.asm
    kernel_tp: u64,
    // 528
    kernel_sp: u64,
    // 536
    kernel_satp: u64,
}

impl TrapFrame {
    pub fn new(hartid: usize) -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            pc: 0,
            kernel_tp: hartid as u64,
            kernel_sp: _kernel_end as u64 - (_stack_size as u64 * hartid as u64),
            kernel_satp: unsafe { KERNEL_SATP } as u64,
        }
    }
}

impl Display for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Registers with hartid={}", self.kernel_tp)?;
        writeln!(f, "ra={:#016x}, sp={:#016x}", self.x[1], self.x[2])?;
        writeln!(f, "gp={:#016x}, tp={:#016x}", self.x[3], self.x[4])?;
        writeln!(f, "fp={:#016x}", self.x[8])?;
        writeln!(f, "a0={:#016x}, a1={:#016x}", self.x[10], self.x[11])?;
        writeln!(f, "a2={:#016x}, a3={:#016x}", self.x[12], self.x[13])?;
        writeln!(f, "a4={:#016x}, a5={:#016x}", self.x[14], self.x[15])?;
        writeln!(f, "a6={:#016x}, a7={:#016x}", self.x[16], self.x[17])?;
        writeln!(f, "sepc={:#x}", self.pc)
        // writeln!(
        //     f,
        //     "sepc={:#x}, satp({})={:#x}",
        //     self.pc,
        //     match (self.satp >> 60) {
        //         0 => "Bare",
        //         8 => "Sv39",
        //         9 => "Sv48",
        //         10 => "Sv57",
        //         11 => "Sv64",
        //         _ => "unimp",
        //     },
        //     self.satp & ((1 << 44) - 1)
        // )
    }
}

pub struct TrapContext {
    // process reference
    // thread reference
}

pub struct EnvironmentCallBody {
    pub function: SystemCall,
}

#[no_mangle]
unsafe fn handle_user_trap(frame: &mut TrapFrame, cause: Scause, _val: usize) -> (usize, Address) {
    let hart = hart::this_hart();
    match cause.cause() {
        Trap::Interrupt(Interrupt::UserTimer) => hart.trap(TrapCause::TimerInterrupt),
        Trap::Interrupt(Interrupt::SupervisorTimer) => todo!("nested interrupt: timer"),
        Trap::Interrupt(Interrupt::UserSoft) => todo!("impossible user soft interrupt"),
        Trap::Exception(exception) => {
            frame.pc += 4;
            match exception {
                Exception::Breakpoint => hart.trap(TrapCause::Breakpoint),
                Exception::UserEnvCall => hart.trap(TrapCause::EnvironmentCall),
                _ => todo!("Unknown exception: {}", cause.bits()),
            }
        }
        _ => {
            unimplemented!("Unknown trap from user: {}", cause.bits())
        }
    }
    hart.arranged_context()
}

#[no_mangle]
unsafe fn handle_kernel_trap(cause: Scause, val: usize) -> (usize, Address){
    // (a0, a1)
    // a0 + a1 > 0 表示跳转到对应的用户空间，否则跳出 kernel trap
    let hart = hart::this_hart();
    match cause.cause() {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            // 从 kernel 进入 user
            hart.clear_ipi();
            return hart.enter_user()
        }
        _ => unimplemented!("Unknown trap from kernel: {}", cause.bits()),
    }
}
