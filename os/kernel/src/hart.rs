use core::arch::asm;

use alloc::vec::Vec;


use crate::{
    rng::lcg::LcGenerator,
    sbi,
    task::sched::unfair::UnfairScheduler,
    timer::{cpu::CpuClock, Timer},
};

use self::app::ApplicationHart;

pub mod app;

pub type TimerImpl = CpuClock;
pub type SchedulerImpl = UnfairScheduler<TimerImpl>;
pub type RandomImpl = LcGenerator;

static mut HARTS: Vec<HartKind> = Vec::new();

#[derive(Debug, PartialEq, Eq)]
pub enum HartStatus {
    Stopped,
    Suspended,
    Started,
}

pub enum HartKind {
    Disabled,
    Application(ApplicationHart<SchedulerImpl, RandomImpl>),
}

pub fn register(hartid: usize, freq: usize) {
    let harts = unsafe { &mut HARTS };
    if hartid > harts.len() {
        let diff = hartid - harts.len();
        for _ in 0..diff {
            harts.push(HartKind::Disabled);
        }
    }
    let timer = TimerImpl::new(freq);
    let seed = timer.uptime();
    let hart = ApplicationHart::new(
        hartid,
        UnfairScheduler::new(hartid, timer),
        RandomImpl::new(seed),
    );
    harts.push(HartKind::Application(hart));
}

pub fn send_ipi(hart_mask: usize) -> bool {
    if let Ok(_) = sbi::send_ipi(hart_mask, 0) {
        true
    } else {
        false
    }
}

pub fn start_all() {
    for i in unsafe { &HARTS } {
        if let HartKind::Application(hart) = i {
            if let Some(HartStatus::Stopped) = hart.get_status() {
                hart.start();
            }
        }
    }
}

pub fn get_hart(id: usize) -> &'static mut HartKind {
    unsafe {
        if id < HARTS.len() {
            &mut HARTS[id as usize]
        } else {
            panic!(
                "reference to hart id {} is out of bound {}",
                id,
                HARTS.len()
            );
        }
    }
}

pub fn hartid() -> usize {
    let mut tp: usize;
    unsafe {
        asm!("mv {tmp}, tp", tmp = out(reg) tp);
    }
    tp
}

pub fn this_hart() -> &'static mut HartKind {
    get_hart(hartid())
}

pub fn enter_user() -> ! {
    if let HartKind::Application(hart) = this_hart() {
        hart.go_awaken()
    } else {
        panic!("hart #{} does not support application mode", hartid())
    }
}
