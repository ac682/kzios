use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{sync::Arc, vec::Vec};
use erhino_shared::{
    proc::Pid,
    sync::{DataLock, InteriorLock},
};
use spin::mutex::SpinMutex;

use crate::{
    sync::{
        hart::{HartLock, HartReadWriteLock},
        spin::{SpinLock, SpinReadWriteLock},
    },
    task::{proc::Process, thread::Thread},
    timer::Timer,
    trap::TrapFrame,
};

use super::Scheduler;

// 使用非 hart lock 意味着不支持嵌套中断，内核期间不可被打断
static mut IDLE_HART_MASK: DataLock<usize, SpinReadWriteLock> =
    DataLock::new(0, SpinReadWriteLock::new());

static mut PROC_TABLE: ProcessTable = ProcessTable::new();

pub struct ProcessTable {
    processes: Vec<ProcessCell>,
    processes_lock: SpinReadWriteLock,
    generation: AtomicUsize,
    pid_generator: AtomicUsize,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            processes_lock: SpinReadWriteLock::new(),
            generation: AtomicUsize::new(0),
            pid_generator: AtomicUsize::new(0),
        }
    }

    pub fn new_pid(&self) -> Pid {
        self.pid_generator.fetch_add(1, Ordering::Relaxed) as Pid
    }
}

pub struct ThreadCell {
    inner: Thread,
    generation: AtomicUsize,
}

pub struct ProcessCell {
    inner: Process,
    generation: AtomicUsize,
    threads: Vec<ThreadCell>,
}

impl ProcessCell {
    pub fn new(pid: Pid, proc: Process, initial_gen: usize) -> Self {
        let mut inner = proc;
        inner.pid = pid;
        Self {
            inner: inner,
            generation: AtomicUsize::new(initial_gen),
            threads: Vec::new(),
        }
    }
}

pub struct FairEnoughScheduler<T: Timer> {
    hartid: usize,
    timer: T,
}

impl<T: Timer> FairEnoughScheduler<T> {
    pub const fn new(id: usize, timer: T) -> Self {
        Self { hartid: id, timer }
    }
}

impl<T: Timer> Scheduler for FairEnoughScheduler<T> {
    fn add(&mut self, proc: Process) {
        let pid = unsafe { PROC_TABLE.new_pid() };
        let cell = ProcessCell::new(pid, proc, unsafe {
            PROC_TABLE.generation.load(Ordering::Relaxed)
        });
        unsafe {
            PROC_TABLE.processes_lock.lock();
            PROC_TABLE.processes.push(cell);
            PROC_TABLE.processes_lock.unlock()
        }
    }

    fn schedule(&mut self) {
        todo!()
    }

    fn context(&self) -> &TrapFrame {
        todo!()
    }
}
