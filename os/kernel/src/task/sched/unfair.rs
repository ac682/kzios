use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{sync::Arc, vec::Vec};
use erhino_shared::{
    proc::Pid,
    sync::{DataLock, InteriorLock, ReadDataLockGuard, ReadWriteDataLock},
};
use spin::mutex::SpinMutex;

use crate::{
    sync::spin::{ReadWriteSpinLock, SpinLock},
    task::{proc::Process, thread::Thread},
    timer::Timer,
    trap::TrapFrame,
};

use super::Scheduler;

// 使用非 hart lock 意味着不支持嵌套中断，内核期间不可被打断
static mut IDLE_HART_MASK: DataLock<usize, SpinLock> = DataLock::new(0, SpinLock::new());

static mut PROC_TABLE: ProcessTable = ProcessTable::new();

pub struct ProcessTable {
    processes: ReadWriteDataLock<Vec<Arc<ProcessCell>>, ReadWriteSpinLock>,
    generation: AtomicUsize,
    pid_generator: AtomicUsize,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            processes: ReadWriteDataLock::new(Vec::new(), ReadWriteSpinLock::new()),
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

impl ThreadCell {
    pub fn new() -> Self {
        todo!()
    }

    pub fn gen(&self) -> usize {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn add_gen(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct ProcessCell {
    inner: Process,
    generation: AtomicUsize,
    threads: ReadWriteDataLock<Vec<Arc<ThreadCell>>, ReadWriteSpinLock>,
}

impl ProcessCell {
    pub fn new(pid: Pid, proc: Process, initial_gen: usize) -> Self {
        let mut inner = proc;
        inner.pid = pid;
        Self {
            inner: inner,
            generation: AtomicUsize::new(initial_gen),
            threads: ReadWriteDataLock::new(Vec::new(), ReadWriteSpinLock::new()),
        }
    }

    pub fn gen(&self) -> usize {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn add_gen(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct FairEnoughScheduler<T: Timer> {
    hartid: usize,
    timer: T,
    proc: Option<Arc<ProcessCell>>,
    thread: Option<Arc<ThreadCell>>,
}

impl<T: Timer> FairEnoughScheduler<T> {
    pub const fn new(id: usize, timer: T) -> Self {
        Self {
            hartid: id,
            timer,
            proc: None,
            thread: None,
        }
    }

    fn pick_process(
        vector: &ReadDataLockGuard<Vec<Arc<ProcessCell>>, ReadWriteSpinLock>,
        gen: usize,
    ) -> Arc<ProcessCell> {
        let scale = vector.len();
        if scale == 0 {
            Self::go_idle()
        }
        for p in vector.iter() {
            if p.gen() * scale < gen {
                return p.clone();
            }
        }
        vector.last().unwrap().clone()
    }

    fn go_idle() -> ! {
        todo!()
    }
}

impl<T: Timer> Scheduler for FairEnoughScheduler<T> {
    fn add(&mut self, proc: Process) {
        let pid = unsafe { PROC_TABLE.new_pid() };
        let mut vector = unsafe { PROC_TABLE.processes.lock_mut() };
        let cell = ProcessCell::new(pid, proc, unsafe {
            PROC_TABLE.generation.load(Ordering::Relaxed) / vector.len()
        });
        vector.push(Arc::new(cell));
    }

    fn schedule(&mut self) {
        let processes = unsafe { PROC_TABLE.processes.lock() };
        let gen = unsafe { PROC_TABLE.generation.load(Ordering::Relaxed) };
        let scale_p = processes.len();
        let mut proc: Arc<ProcessCell> = if let Some(p) = &self.proc {
            if p.gen() * scale_p < gen {
                p.add_gen();
                p.clone()
            } else {
                Self::pick_process(&processes, gen)
            }
        } else {
            Self::pick_process(&processes, gen)
        };
        let mut thread: Option<Arc<ThreadCell>> = None;
    }

    fn context(&self) -> &TrapFrame {
        todo!()
    }
}
