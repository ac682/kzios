use alloc::sync::Arc;
use core::arch::global_asm;
use core::ptr::null_mut;
use riscv::register::mcause::{Exception, Mcause, Trap};
use riscv::register::mstatus::Mstatus;
global_asm!(include_str!("trap.S"));

use crate::println;
use riscv::register::mtvec::TrapMode;
use riscv::register::{mepc, mscratch, mtvec};

extern "C" {
    fn _m_trap_vector();
}

static mut KERNEL_TRAP: TrapFrame = TrapFrame::new();

pub fn init() {
    unsafe {
        mscratch::write(&KERNEL_TRAP as *const TrapFrame as usize);
        mtvec::write(_m_trap_vector as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub extern "C" fn handle_machine_trap(
    epc: usize,
    tval: usize,
    cause: Mcause,
    hart_id: usize,
    status: Mstatus,
    frame: &mut TrapFrame,
) {
    if cause.is_exception() {
        let mut mepc = mepc::read();
        mepc += 8;
        mepc::write(mepc);
    }

    match cause.cause() {
        Trap::Exception(Exception::Breakpoint) => println!("break"),
        Trap::Exception(Exception::LoadPageFault) => println!("page fault"),

        _ => println!("unknown"),
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub x: [usize; 32], // 0-255
    pub f: [usize; 32],
    pub satp: usize,
    pub trap_stack: *mut u8,
}

impl TrapFrame {
    pub const fn new() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            satp: 0,
            trap_stack: null_mut(),
        }
    }
}
