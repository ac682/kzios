use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, string::ToString, sync::Arc};
use erhino_shared::proc::{ExitCode, Pid, Tid};

use crate::{
    println,
    proc::{thread::Thread, Process},
    sync::{hart::HartReadWriteLock, DataLock, InteriorLock, InteriorReadWriteLock},
};

use super::Scheduler;

static mut PROC_TABLE: ProcessTable = ProcessTable::new();

pub struct ProcessCell {
    proc: Process,
    generation: AtomicUsize,
    // 不可以一条线程也没有哦
    head: Option<Arc<UnsafeCell<ThreadCell>>>,
    head_lock: HartReadWriteLock,
    next: Option<Arc<UnsafeCell<ProcessCell>>>,
    next_lock: HartReadWriteLock,
}

impl ProcessCell {
    pub fn new(value: Process, initial_generation: usize) -> Self {
        let cell = ThreadCell::new(Thread::new("main".to_string()), initial_generation);
        Self {
            proc: value,
            generation: AtomicUsize::new(initial_generation),
            head: Some(Arc::new(UnsafeCell::new(cell))),
            head_lock: HartReadWriteLock::new(),
            next: None,
            next_lock: HartReadWriteLock::new(),
        }
    }
}

pub struct ThreadCell {
    thread: Thread,
    generation: AtomicUsize,
    next: Option<Arc<UnsafeCell<ThreadCell>>>,
    next_lock: HartReadWriteLock,
}

impl ThreadCell {
    pub fn new(thread: Thread, initial_generation: usize) -> Self {
        Self {
            thread,
            generation: AtomicUsize::new(initial_generation),
            next: None,
            next_lock: HartReadWriteLock::new(),
        }
    }
}

pub struct ProcessTable {
    generation: AtomicUsize,
    current_pid: AtomicUsize,
    // 用链表是因为可以单独上锁，修改的时候只需要锁前中两个元素就行
    head: Option<Arc<UnsafeCell<ProcessCell>>>,
    head_lock: HartReadWriteLock,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            generation: AtomicUsize::new(0),
            current_pid: AtomicUsize::new(0),
            head: None,
            head_lock: HartReadWriteLock::new(),
        }
    }

    pub fn add(&mut self, mut proc: Process) -> Pid {
        let pid = self.current_pid.fetch_add(1, Ordering::Relaxed) as Pid;
        proc.pid = pid;
        let cell = ProcessCell::new(proc, self.generation.load(Ordering::Relaxed));
        if let Some(mut node_ptr) = {
            self.head_lock.lock();
            let ptr = self.head.as_ref();
            self.head_lock.unlock();
            ptr
        } {
            while let Some(next) = {
                let node = unsafe { &*node_ptr.get() };
                node.next_lock.lock();
                let after = &node.next;
                node.next_lock.unlock();
                after
            } {
                node_ptr = next;
            }
            let node = unsafe { &mut *node_ptr.get() };
            node.next_lock.lock_mut();
            node.next = Some(Arc::new(UnsafeCell::new(cell)));
            node.next_lock.unlock();
        } else {
            self.head_lock.lock_mut();
            self.head = Some(Arc::new(UnsafeCell::new(cell)));
            self.head_lock.unlock();
        }
        pid
    }
}

pub struct SmoothScheduler {
    hartid: usize,
    current: Option<(Arc<UnsafeCell<ProcessCell>>, Arc<UnsafeCell<ThreadCell>>)>,
}

impl SmoothScheduler {
    pub fn new(hartid: usize) -> Self {
        Self {
            hartid,
            current: None,
        }
    }
}

impl Scheduler for SmoothScheduler {
    fn add(&self, proc: Process) -> Pid {
        unsafe { PROC_TABLE.add(proc) }
    }

    fn tick(&mut self) -> (Pid, Tid) {
        // CFS 里是基于时间，会相差很大，因此要求有序以保证最少时间者优先。这里是访问次数，不会相差大于个位数，让某几个线程领先个位数次访问问题也不大
        if unsafe { PROC_TABLE.head.is_none() } {
            panic!("no processes at all");
        }
        loop {
            let table_generation = unsafe { PROC_TABLE.generation.load(Ordering::Relaxed) };
            // TODO: check if current process is still behind other's generation then pick another thread but the same process
            let mut proc_option = unsafe {
                PROC_TABLE.head_lock.lock();
                let ptr = PROC_TABLE.head.as_ref();
                PROC_TABLE.head_lock.unlock();
                ptr
            };
            let mut max_proc_generation = table_generation;
            while let Some(proc_ptr) = proc_option {
                let proc = unsafe { &*proc_ptr.get() };
                let proc_generation = proc.generation.load(Ordering::Relaxed);
                if proc_generation <= table_generation {
                    // selected
                    let mut max_thread_generation = proc_generation;
                    let mut thread_option = {
                        proc.head_lock.lock();
                        let ptr = proc.head.as_ref();
                        proc.head_lock.unlock();
                        ptr
                    };
                    while let Some(thread_ptr) = thread_option {
                        let thread = unsafe { &*thread_ptr.get() };
                        let thread_generation = thread.generation.load(Ordering::Relaxed);
                        if thread_generation <= proc_generation {
                            thread.generation.fetch_add(1, Ordering::Relaxed);
                            // set as current
                            self.current = Some((proc_ptr.clone(), thread_ptr.clone()));
                            return (proc.proc.pid, thread.thread.tid);
                        } else {
                            max_thread_generation = if thread_generation > max_thread_generation {
                                thread_generation
                            } else {
                                max_thread_generation
                            };

                            thread.next_lock.lock();
                            thread_option = thread.next.as_ref();
                            thread.next_lock.unlock();
                        }
                    }
                    // update proc generation and continue looking for next proc
                    proc.generation
                        .fetch_max(max_thread_generation, Ordering::Relaxed);
                } else {
                    max_proc_generation = if proc_generation > max_proc_generation {
                        proc_generation
                    } else {
                        max_proc_generation
                    };

                    proc.next_lock.lock();
                    proc_option = proc.next.as_ref();
                    proc.next_lock.unlock();
                }
            }
            // update table generation and schedule again
            unsafe {
                PROC_TABLE
                    .generation
                    .fetch_max(max_proc_generation, Ordering::Relaxed)
            };
        }
    }

    fn begin(&mut self) {
        loop {
            let (pid, tid) = self.tick();
            println!("switched to {}:{}", pid, tid);
            for _ in 0..500_0000 {}
        }
    }

    fn current(&mut self) -> Option<&mut Process> {
        todo!()
    }

    fn find(&mut self, pid: Pid) -> Option<&Process> {
        todo!()
    }

    fn find_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        todo!()
    }

    fn finish(&mut self, code: ExitCode) {
        todo!()
    }

    fn kill(&mut self, pid: Pid) {
        todo!()
    }
}
