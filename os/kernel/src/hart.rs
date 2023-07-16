use alloc::vec::Vec;
use riscv::register::{sscratch, sstatus, stvec, utvec::TrapMode};

use crate::{
    external::{_hart_num, _trap_vector},
    sbi,
    trap::TrapFrame,
};

static mut HARTS: Vec<Hart> = Vec::new();

pub struct Hart {
    id: usize,
    frame: TrapFrame,
}

impl Hart {
    pub const fn new(hartid: usize) -> Self {
        Self {
            id: hartid,
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
        if let Ok(_) = sbi::send_ipi(1 << self.id) {
            true
        } else {
            false
        }
    }
}

pub fn of_hart(id: usize) -> &'static mut Hart {
    unsafe {
        if id < HARTS.len() {
            &mut HARTS[id as usize]
        } else {
            panic!("reference to hart id {} is out of bound", id);
        }
    }
}

pub fn init(freq: usize) {
    unsafe {
        for i in 0..(_hart_num as usize) {
            HARTS.push(Hart::new(i));
        }
    }
}

pub fn send_ipi(hart_mask: usize) -> bool {
    if let Ok(_) = sbi::send_ipi(hart_mask) {
        true
    } else {
        false
    }
}
