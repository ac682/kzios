use alloc::vec::Vec;

use crate::{external::_hart_num, sync::hart::HartReadWriteLock};

use self::flat::FlatScheduler;

use super::Process;

pub mod flat;

type SchedulerImpl = FlatScheduler;

static mut SCHEDULERS: Vec<SchedulerImpl> = Vec::new();

pub trait Scheduler {
    fn new() -> Self;
    fn tick(&self);
}

pub fn init() {
    unsafe {
        for i in 0..(_hart_num as usize) {
            SCHEDULERS.push(SchedulerImpl::new());
        }
    }
}
