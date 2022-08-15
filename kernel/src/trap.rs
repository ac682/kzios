use alloc::sync::Arc;
use core::arch::global_asm;
use core::ptr::null_mut;

use riscv::register::{mcause, mepc, mscratch, mtvec};
use riscv::register::mcause::{Exception, Interrupt, Mcause, Trap};
use riscv::register::mstatus::Mstatus;
use riscv::register::mtvec::TrapMode;

use crate::println;
use crate::process::manager::schedule_next_process;
use crate::syscall::forward;
use crate::timer::set_next_timer;

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
pub fn handle_machine_trap(frame: *mut TrapFrame) -> usize {
    let cause = mcause::read();
    match cause.cause() {
        Trap::Exception(Exception::InstructionFault) => println!("inst access fault"),
        Trap::Exception(Exception::Breakpoint) => println!("break"),
        Trap::Exception(Exception::InstructionPageFault) => println!("inst page fault"),
        Trap::Exception(Exception::LoadPageFault) => println!("page fault"),
        Trap::Exception(Exception::UserEnvCall) => {
            unsafe {
                let frame = *frame;
                let id = frame.x[17];
                let arg0 = frame.x[10];
                let arg1 = frame.x[11];
                let arg2 = frame.x[12];
                let arg3 = frame.x[13];
                forward(id, arg0, arg1, arg2, arg3);
            }
        }
        Trap::Exception(Exception::SupervisorEnvCall) => println!("supervisor ecall"),
        Trap::Exception(Exception::MachineEnvCall) => println!("machine ecall"),
        Trap::Interrupt(Interrupt::MachineTimer) => {
            set_next_timer();
            schedule_next_process();
        }
        _ => println!("unknown"),
    }
    let epc = mepc::read();
    epc + if cause.is_exception() { 8 } else { 0 }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    // 0-255
    pub x: [usize; 32],
    // 256 - 511
    pub f: [usize; 32],
    // 512-519
    pub satp: usize,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0,
        }
    }
}
