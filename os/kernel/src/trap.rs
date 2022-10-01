use alloc::sync::Arc;
use core::arch::global_asm;
use core::fmt::Display;
use core::ptr::null_mut;

use riscv::register::{mcause, mepc, mscratch, mtvec};
use riscv::register::mcause::{Exception, Interrupt, Mcause, Trap};
use riscv::register::mstatus::Mstatus;
use riscv::register::mtvec::TrapMode;

use crate::{println, switch_to_user, timer};
use crate::process::scheduler::forward_tick;
use crate::syscall::forward;
use crate::timer::set_next_timer;
use crate::utils::calculate_instruction_length;

extern "C" {
    fn _m_trap_vector();
}

static mut KERNEL_TRAP: TrapFrame = TrapFrame::zero();

pub fn init() {
    unsafe {
        mscratch::write(&KERNEL_TRAP as *const TrapFrame as usize);
        mtvec::write(_m_trap_vector as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub extern "C" fn handle_machine_trap(frame: &mut TrapFrame) {
    let cause = mcause::read();
    match cause.cause() {
        Trap::Exception(Exception::MachineEnvCall) => unsafe {
            let which = frame.x[10];
            match which {
                0 => {
                    // enter userspace
                    switch_to_user();
                }
                _ => (),
            };
        },
        Trap::Exception(Exception::UserEnvCall) => unsafe {
            let id = frame.x[17];
            let arg0 = frame.x[10];
            let arg1 = frame.x[11];
            let arg2 = frame.x[12];
            let arg3 = frame.x[13];
            forward(id, arg0, arg1, arg2, arg3);
            frame.pc += 4;
        },
        Trap::Exception(Exception::StorePageFault) => {
            panic!("Store/AMO Page Fault, mepc={:#x}", frame.pc);
        }
        Trap::Exception(Exception::LoadPageFault) => {
            panic!("Load Page Fault. How to know which page");
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            panic!("Illegal Instruction");
        }
        Trap::Exception(Exception::InstructionFault) => {
            panic!("Instruction Fault");
        }
        Trap::Exception(Exception::InstructionPageFault) => {
            panic!("Instruction Page Fault with mepc={:#x}", frame.pc);
        }
        Trap::Exception(Exception::Breakpoint) => {
            // TODO: dump frame
            println!("break!");
            frame.pc += 4;
        }
        Trap::Interrupt(Interrupt::MachineTimer) => {
            forward_tick();
        }
        _ => panic!(
            "unknown trap cause: {:#b}({})\nTrapFrame:\n{}",
            cause.bits(),
            cause.bits(),
            frame
        ),
    };
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
    pub status: u64,
    // 528-535
    pub pc: u64,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0,
            status: 0,
            pc: 0,
        }
    }
}

impl Display for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!("dump trap frame")
    }
}
