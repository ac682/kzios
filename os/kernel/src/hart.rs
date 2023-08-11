use core::arch::asm;

use alloc::vec::Vec;
use erhino_shared::{mem::Address, proc::Tid};
use riscv::register::{scause::Scause, sip, sscratch, sstatus, stvec, utvec::TrapMode};

use crate::{
    external::{_hart_num, _switch},
    println, sbi,
    sync::hart,
    task::{
        proc::Process,
        sched::{unfair::UnfairScheduler, Scheduler},
    },
    timer::{hart::HartTimer, Timer},
    trap::{TrapCause, TrapFrame},
};

type TimerImpl = HartTimer;
type SchedulerImpl = UnfairScheduler;

static mut HARTS: Vec<Hart<TimerImpl, SchedulerImpl>> = Vec::new();

pub struct Hart<T: Timer, S: Scheduler> {
    id: usize,
    timer: T,
    scheduler: S,
}

impl<T: Timer, S: Scheduler> Hart<T, S> {
    pub const fn new(hartid: usize, timer: T, scheduler: S) -> Self {
        Self {
            id: hartid,
            timer,
            scheduler,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn arranged_context(&self) -> (usize, Address) {
        let (p, _, trapframe, _) = self.scheduler.context();
        (p.memory.satp(), trapframe)
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

    pub fn enter_user(&mut self) -> ! {
        self.scheduler.schedule();
        let (p,_, trapframe, trampoline) = self.scheduler.context();
        unsafe{
            _switch(trampoline, p.memory.satp(), trapframe)
        }
    }

    pub fn trap(&mut self, cause: TrapCause) {
        match cause {
            TrapCause::Breakpoint => {
                println!("#{} Pid={} requested a breakpoint", self.id, "unimp");
            }
            TrapCause::TimerInterrupt => {
                self.scheduler.schedule();
                self.timer.schedule_next(self.scheduler.next_timeslice());
            }
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
                TimerImpl::new(
                    i,
                    if i < freq.len() {
                        freq[i]
                    } else {
                        freq[freq.len() - 1]
                    },
                ),
                SchedulerImpl::new(i),
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

pub fn get_hart(id: usize) -> &'static mut Hart<TimerImpl, SchedulerImpl> {
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

pub fn this_hart() -> &'static mut Hart<TimerImpl, SchedulerImpl> {
    get_hart(hartid())
}

pub fn add_process(proc: Process) {
    println!("kernel process added: {}", proc.name);
    this_hart().scheduler.add(proc, None);
}
