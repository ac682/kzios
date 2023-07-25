use core::{cell::UnsafeCell, sync::atomic::AtomicUsize};

use alloc::{sync::Arc, vec::Vec};

use crate::{
    sync::{
        hart::{HartLock, HartReadWriteLock},
        spin::SpinLock,
        DataLock,
    },
    task::{proc::Process, thread::Thread},
    timer::Timer,
};

use super::Scheduler;

pub struct ThreadCell {
    inner: Thread,
    generation: usize,
    lock: SpinLock,
}

pub struct ProcessCell {
    inner: Process,
    generation: usize,
    threads: Vec<ThreadCell>,
    lock: SpinLock,
}

pub struct SmoothScheduler<T: Timer> {
    hartid: usize,
    timer: T,
    processes: Vec<ProcessCell>,
    max_generation: usize,
}

impl<T: Timer> SmoothScheduler<T> {
    pub const fn new(id: usize, timer: T) -> Self {
        Self {
            hartid: id,
            timer,
            processes: Vec::new(),
            max_generation: 0,
        }
    }
}

impl<T: Timer> Scheduler for SmoothScheduler<T> {
    fn add(&mut self) {
        todo!()
    }

    fn schedule(&mut self) {
        todo!()
    }
}
