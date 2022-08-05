use alloc::sync::Arc;
use core::arch::global_asm;
use core::ptr::null_mut;
use riscv::register::mcause::{Exception, Interrupt, Mcause, Trap};
use riscv::register::mstatus::Mstatus;

use crate::println;
use crate::process::manager::schedule;
use riscv::register::mtvec::TrapMode;
use riscv::register::{mcause, mepc, mscratch, mtvec};

extern "C" {
    fn _m_trap_vector();

    fn _trap_stack_end();
}

static mut KERNEL_TRAP: TrapFrame = TrapFrame::zero();

pub fn init() {
    unsafe {
        mscratch::write(&KERNEL_TRAP as *const TrapFrame as usize);
        mtvec::write(_m_trap_vector as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn handle_machine_trap() {
    let cause = mcause::read();
    match cause.cause() {
        Trap::Exception(Exception::InstructionFault) => println!("inst access fault"),
        Trap::Exception(Exception::Breakpoint) => println!("break"),
        Trap::Exception(Exception::InstructionPageFault) => println!("inst page fault"),
        Trap::Exception(Exception::LoadPageFault) => println!("page fault"),
        Trap::Exception(Exception::UserEnvCall) => println!("user ecall"),
        Trap::Exception(Exception::SupervisorEnvCall) => println!("supervisor ecall"),
        Trap::Exception(Exception::MachineEnvCall) => println!("machine ecall"),
        Trap::Interrupt(Interrupt::MachineTimer) => schedule(),
        _ => println!("unknown"),
    }

    // TODO: this should be done in asm
    let epc = mepc::read();
    if cause.is_exception() {
        mepc::write(epc + 8);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub x: [usize; 32], // 0-255
    pub f: [usize; 32], // 256 - 511
    pub satp: usize,    // 512-519
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0
        }
    }
}
