use core::fmt::Display;

use erhino_shared::{call::SystemCall, mem::Address};
use riscv::register::scause::{Exception, Interrupt, Scause, Trap};

use crate::{
    external::{_kernel_end, _stack_size, _kernel_trap},
    hart,
    mm::{MemoryOperation, KERNEL_SATP},
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
    PageFault(Address, MemoryOperation),
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
    // 以下是临时数据，与 TrapFrame 所属的进程无关，只读
    // 520 Currently the hart it running in. Guaranteed by trap_vector in assembly.asm
    kernel_tp: u64,
    // 528
    kernel_sp: u64,
    // 536
    kernel_satp: u64,
    // 544
    kernel_trap: u64,
    // 552
    user_trap: u64,
}

impl TrapFrame {
    pub fn init(
        &mut self,
        hartid: usize,
        entry_point: Address,
        stack_address: Address,
        user_trap: Address,
    ) {
        self.x = [0; 32];
        self.f = [0; 32];
        self.x[2] = stack_address as u64;
        self.pc = entry_point as u64;
        self.kernel_tp = hartid as u64;
        self.kernel_sp = _kernel_end as u64 - (_stack_size as u64 * hartid as u64);
        self.kernel_satp = unsafe { KERNEL_SATP } as u64;
        self.kernel_trap = _kernel_trap as u64;
        self.user_trap = user_trap as u64;
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
unsafe fn handle_kernel_trap(cause: Scause, val: usize) {
    let hart = hart::this_hart();
    match cause.cause() {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            // 从 kernel 进入 user
            hart.clear_ipi();
            hart.enter_user();
        }
        _ => unimplemented!("Unknown trap from kernel: {}", cause.bits()),
    }
}

#[no_mangle]
unsafe fn handle_user_trap(cause: Scause, val: usize) -> (usize, Address) {
    let hart = hart::this_hart();
    match cause.cause() {
        Trap::Interrupt(Interrupt::UserTimer) => hart.trap(TrapCause::TimerInterrupt),
        Trap::Interrupt(Interrupt::SupervisorTimer) => todo!("nested interrupt: timer"),
        Trap::Interrupt(Interrupt::UserSoft) => todo!("impossible user soft interrupt"),
        Trap::Exception(exception) => {
            match exception {
                Exception::Breakpoint => hart.trap(TrapCause::Breakpoint),
                Exception::UserEnvCall => hart.trap(TrapCause::EnvironmentCall),
                Exception::LoadPageFault => {
                    hart.trap(TrapCause::PageFault(val as Address, MemoryOperation::Read))
                }
                Exception::StorePageFault => {
                    hart.trap(TrapCause::PageFault(val as Address, MemoryOperation::Write))
                }
                Exception::InstructionPageFault => hart.trap(TrapCause::PageFault(
                    val as Address,
                    MemoryOperation::Execute,
                )),
                _ => todo!("unknown exception: {}", cause.bits()),
            }
        }
        _ => {
            unimplemented!("unknown trap from user: {}", cause.bits())
        }
    }
    hart.arranged_context()
}
