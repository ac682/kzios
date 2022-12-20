use core::{
    cell::{RefCell, UnsafeCell},
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{rc::Rc, string::ToString, sync::Arc};
use erhino_shared::proc::{ExitCode, Pid, ProcessState, Tid};

use crate::{
    proc::{thread::Thread, Process},
    sync::{
        hart::{HartLock, HartReadWriteLock},
        InteriorLock, InteriorReadWriteLock,
    },
    timer::Timer,
};

use super::Scheduler;

static mut PROC_TABLE: ProcessTable = ProcessTable::new();

pub struct ProcessCell {
    proc: Process,
    generation: AtomicUsize,
    head: Option<Arc<UnsafeCell<ThreadCell>>>,
    head_lock: HartReadWriteLock,
    next: Option<Arc<UnsafeCell<ProcessCell>>>,
    next_lock: HartReadWriteLock,
}

impl ProcessCell {
    pub fn new(value: Process, initial_generation: usize) -> Self {
        let main = Thread::new(
            "main".to_string(),
            value.entry_point,
            0x40_0000_0000,
            value.memory.satp(),
        );
        let cell = ThreadCell::new(main, initial_generation);
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
    // must acquire lock before modify the Thread data or put it in hart context
    state_lock: HartLock,
}

impl ThreadCell {
    pub fn new(thread: Thread, initial_generation: usize) -> Self {
        Self {
            thread,
            generation: AtomicUsize::new(initial_generation),
            next: None,
            next_lock: HartReadWriteLock::new(),
            state_lock: HartLock::new(),
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

pub struct SmoothScheduler<T: Timer> {
    hartid: usize,
    timer: Rc<RefCell<T>>,
    current: Option<(Arc<UnsafeCell<ProcessCell>>, Arc<UnsafeCell<ThreadCell>>)>,
}

impl<T: Timer> SmoothScheduler<T> {
    pub fn new(hartid: usize, timer: Rc<RefCell<T>>) -> Self {
        Self {
            hartid,
            timer,
            current: None,
        }
    }

    pub fn switch_next(&mut self) -> (Pid, Tid) {
        // CFS 里是基于时间，会相差很大，因此要求有序以保证最少时间者优先。这里是访问次数，不会相差大于个位数，让某几个线程领先个位数次访问问题也不大
        if unsafe { PROC_TABLE.head.is_none() } {
            panic!("no processes at all");
        }
        if let Some((current_process, current_thread)) = self.current.take() {
            // check if current process is still behind other's generation then pick another thread but the same process and do not unlock
            let table_generation = unsafe { PROC_TABLE.generation.load(Ordering::Relaxed) };
            let proc = unsafe { &mut *current_process.get() };
            let thread = unsafe { &mut *current_thread.get() };
            let proc_generation = proc.generation.load(Ordering::Relaxed);
            let thread_generation = thread.generation.load(Ordering::Relaxed);
            if table_generation >= proc_generation {
                if proc_generation >= thread_generation {
                    // do nothing but put them back
                    thread.generation.fetch_add(1, Ordering::Relaxed);
                    self.current = Some((current_process, current_thread));
                    return (proc.proc.pid, thread.thread.tid);
                } else {
                    thread.thread.state = ProcessState::Ready;
                    thread.state_lock.unlock();
                    let new_thread_ptr = self.pick_thread(proc);
                    let new_thread = unsafe { &mut *new_thread_ptr.get() };
                    new_thread.thread.state = ProcessState::Running;
                    let pid = proc.proc.pid;
                    let tid = new_thread.thread.tid;
                    self.current = Some((current_process, new_thread_ptr));
                    return (pid, tid);
                }
            } else {
                thread.thread.state = ProcessState::Ready;
                thread.state_lock.unlock();
                let proc_ptr = self.pick_process();
                let proc = unsafe { &mut *proc_ptr.get() };
                let pid = proc.proc.pid;
                let new_thread_ptr = self.pick_thread(proc);
                let new_thread = unsafe { &mut *new_thread_ptr.get() };
                new_thread.thread.state = ProcessState::Running;
                let tid = new_thread.thread.tid;
                self.current = Some((proc_ptr, new_thread_ptr));
                return (pid, tid);
            }
        } else {
            let proc_ptr = self.pick_process();
            let proc = unsafe { &mut *proc_ptr.get() };
            let pid = proc.proc.pid;
            let new_thread_ptr = self.pick_thread(proc);
            let new_thread = unsafe { &mut *new_thread_ptr.get() };
            new_thread.thread.state = ProcessState::Running;
            let tid = new_thread.thread.tid;
            self.current = Some((proc_ptr, new_thread_ptr));
            return (pid, tid);
        }
    }

    fn pick_process(&mut self) -> Arc<UnsafeCell<ProcessCell>> {
        loop {
            let table_generation = unsafe { PROC_TABLE.generation.load(Ordering::Relaxed) };
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
                    // println!("PID: {} picked for {}", proc.proc.pid, proc_generation);
                    return proc_ptr.clone();
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

    fn pick_thread(&self, proc: &mut ProcessCell) -> Arc<UnsafeCell<ThreadCell>> {
        // selected
        loop {
            let proc_generation = proc.generation.load(Ordering::Relaxed);
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
                if thread_generation <= proc_generation && thread.state_lock.try_lock() {
                    if thread.thread.state == ProcessState::Ready {
                        thread.generation.fetch_add(1, Ordering::Relaxed);
                        // println!(
                        //     "TID: {} picked for {}",
                        //     thread.thread.tid, thread_generation
                        // );
                        return thread_ptr.clone();
                    }
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
        }
    }
}

impl<T: Timer> Scheduler for SmoothScheduler<T> {
    fn add(&self, proc: Process) -> Pid {
        unsafe { PROC_TABLE.add(proc) }
    }

    fn tick(&mut self) -> (Pid, Tid) {
        let res = self.switch_next();
        let mut timer = self.timer.borrow_mut();
        let cycles = timer.ms_to_cycles(50);
        timer.set_timer(cycles);
        res
    }

    fn begin(&mut self) {
        self.tick();
    }

    fn current(&mut self) -> Option<(&mut Process, &mut Thread)> {
        if let Some((proc_ptr, thread_ptr)) = self.current.as_ref() {
            Some(unsafe { (&mut (*proc_ptr.get()).proc, &mut (*thread_ptr.get()).thread) })
        } else {
            None
        }
    }

    fn find(&mut self, _pid: Pid) -> Option<&Process> {
        todo!()
    }

    fn find_mut(&mut self, _pid: Pid) -> Option<&mut Process> {
        todo!()
    }

    fn finish(&mut self, _code: ExitCode) {
        // 这里是错的，线程死亡应该有自己的方式，这里应该标记进程的状态
        if let Some((_, thread_ptr)) = self.current.as_ref() {
            let thread = unsafe { &mut *thread_ptr.get() };
            thread.thread.state = ProcessState::Dead;
        }
    }

    fn kill(&mut self, _pid: Pid) {
        todo!()
    }
}
