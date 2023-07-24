use core::arch::asm;

use alloc::vec::Vec;
use riscv::register::{scause::Scause, sip, sscratch, sstatus, stvec, utvec::TrapMode};

use crate::{
    external::{_hart_num, _trap_vector},
    println, sbi,
    trap::{TrapCause, TrapFrame},
};

use self::timer::HartTimer;

pub mod timer;

static mut HARTS: Vec<Hart> = Vec::new();

pub struct Hart {
    id: usize,
    timer: HartTimer,
    frame: TrapFrame,
}

impl Hart {
    pub const fn new(hartid: usize, freq: usize) -> Self {
        Self {
            id: hartid,
            timer: HartTimer::new(hartid, freq),
            frame: TrapFrame::new(hartid),
        }
    }

    pub fn init(&mut self) {
        // call on boot by current hart
        // setup trap & interrupts
        unsafe {
            stvec::write(_trap_vector as usize, TrapMode::Direct);
            sscratch::write(&self.frame as *const TrapFrame as usize);
            sstatus::set_fs(sstatus::FS::Initial);
            sstatus::set_sie();
        }
    }

    pub fn send_ipi(&self) -> bool {
        if let Ok(_) = sbi::send_ipi(1, self.id as isize) {
            true
        } else {
            false
        }
    }

    pub fn clear_ipi(&self) {
        // clear sip.SSIP => sip[1] = 0
        let mut sip = 0usize;
        unsafe { asm!("csrr {o}, sip", "csrw sip, {i}", o = out(reg) sip, i = in(reg) sip & !2) }
    }

    pub fn handle_user_trap(&mut self, cause: TrapCause) -> &TrapFrame {
        match cause {
            TrapCause::Breakpoint => {
                println!("#{} Pid={} requested a breakpoint", self.id, "unimp");
                &self.frame
            },
            _ => {
                todo!("unimplemented trap cause {:?}", cause)
            }
        }
    }
}

pub fn init(freq: &[usize]) {
    unsafe {
        for i in 0..(_hart_num as usize) {
            HARTS.push(Hart::new(
                i,
                if i < freq.len() {
                    freq[i]
                } else {
                    freq[freq.len() - 1]
                },
            ));
        }
    }
}

pub fn send_ipi(hart_mask: usize) -> bool {
    if let Ok(_) = sbi::send_ipi(hart_mask, 0) {
        true
    } else {
        false
    }
}

pub fn send_ipi_all() -> bool {
    if let Ok(_) = sbi::send_ipi(0, -1) {
        true
    } else {
        false
    }
}

pub fn get_hart(id: usize) -> &'static mut Hart {
    unsafe {
        if id < HARTS.len() {
            &mut HARTS[id as usize]
        } else {
            panic!("reference to hart id {} is out of bound", id);
        }
    }
}

pub fn hartid() -> usize {
    let mut tp: usize = 0;
    unsafe {
        asm!("mv {tmp}, tp", tmp = out(reg) tp);
    }
    tp
}

pub fn this_hart() -> &'static mut Hart {
    get_hart(hartid())
}
