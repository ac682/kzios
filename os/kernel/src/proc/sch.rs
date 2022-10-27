use alloc::vec::Vec;
use riscv::register::mhartid;

use crate::{external::_hart_num, sync::hart::HartReadWriteLock, timer};

use self::flat::FlatScheduler;

use super::Process;

pub mod flat;

type SchedulerImpl = FlatScheduler;

// 一个 hart 用一个调度器，不会抢占资源
static mut SCHEDULERS: Vec<SchedulerImpl> = Vec::new();

pub trait Scheduler {
    fn new(hartid: usize) -> Self;
    fn add(proc: Process);
    fn tick(&mut self);
    fn begin(&mut self);
    fn current(&mut self) -> Option<&mut Process>;
}

pub fn init() {
    unsafe {
        for i in 0..(_hart_num as usize) {
            SCHEDULERS.push(SchedulerImpl::new(i));
        }
    }
}

pub fn add_process(proc: Process){
    SchedulerImpl::add(proc);
}

pub fn enter_user_mode(hartid: usize){
    unsafe{
        SCHEDULERS[hartid].begin();
    }
}

pub fn forward_tick(){
    let hartid = mhartid::read();
    unsafe{
        SCHEDULERS[hartid].tick();
    }
}

pub fn current_process(hartid: usize) -> Option<&'static mut Process>{
    unsafe{
        SCHEDULERS[hartid].current()
    }
}