use alloc::sync::Arc;
use core::arch::global_asm;
use core::ptr::null_mut;

use riscv::register::{mcause, mepc, mscratch, mtvec};
use riscv::register::mcause::{Exception, Interrupt, Mcause, Trap};
use riscv::register::mstatus::Mstatus;
use riscv::register::mtvec::TrapMode;

use crate::{println, timer};
use crate::process::scheduler::forward_tick;
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
pub extern "C" fn handle_machine_trap(frame: *const TrapFrame, epc: usize) {
    let cause = mcause::read();
    match cause.cause() {
        Trap::Exception(Exception::UserEnvCall) => unsafe {
            let frame = *frame;
            let id = frame.x[17];
            let arg0 = frame.x[10];
            let arg1 = frame.x[11];
            let arg2 = frame.x[12];
            let arg3 = frame.x[13];
            forward(id, arg0, arg1, arg2, arg3)
        },
        Trap::Interrupt(Interrupt::MachineTimer) => {
            forward_tick();
        }
        _ => panic!("unknown trap cause"),
    };
    if cause.is_exception() {
        let new_mepc = mepc::read();
        if new_mepc == epc {
            mepc::write(epc + 4);
        }
    }
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
    // 520-527
    pub status: usize,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0,
            status: 0,
        }
    }
}
