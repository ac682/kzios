use core::fmt::Display;

use riscv::register::{mcause::Mcause, mscratch, mtvec, utvec::TrapMode};

extern "C" {
    fn _trap_vector();
}

static mut KERNEL_TRAP: TrapFrame = TrapFrame::zero();

pub fn init() {
    unsafe {
        mscratch::write(&KERNEL_TRAP as *const TrapFrame as usize);
        mtvec::write(_trap_vector as usize, TrapMode::Direct);
    }
}

#[no_mangle]
fn handle_trap(cause: Mcause, _frame: &mut TrapFrame) {
    match cause.cause() {
        _ => panic!("unknown trap cause: {}", cause.bits()),
    }
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
