use core::{arch::global_asm, panic};

use alloc::boxed::Box;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sepc, sscratch,
    sstatus::{self, Sstatus},
    stvec,
};

global_asm!(include_str!("trap.S"));

extern "C" {
    fn trap_vector();
}

#[repr(C)]
pub struct TrapFrame {
    x: [usize; 32],
    fx: [usize; 32],
    satp: usize,
}

impl TrapFrame {
    pub fn new() -> Self {
        Self {
            x: [0; 32],
            fx: [0; 32],
            satp: 0,
        }
    }
}

pub fn init() {
    // sscratch 指向 TrapFrame 所在地址，用以储存寄存器
    let frame = TrapFrame::new();
    sscratch::write(&frame as *const TrapFrame as usize);
    unsafe {
        stvec::write(trap_vector as usize, TrapMode::Direct);
    }
}

// trap_vector 跳转到这
#[no_mangle]
extern "C" fn handle_trap() {
    let cause = scause::read();
    if cause.is_exception() {
        let mut sepc = sepc::read();
        sepc += 8;
        sepc::write(sepc);
    }
    match cause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => println!("TIMER TICK!"),
        Trap::Exception(Exception::Breakpoint) => println!("BREAKPOINT!"),
        Trap::Exception(Exception::LoadPageFault) => panic!("LOAD PAGE FAULT"),
        _ => panic!("UNKNOWN TRAP!"),
    };
}
