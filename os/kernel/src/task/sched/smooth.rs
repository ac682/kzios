use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{sync::Arc, vec::Vec};
use erhino_shared::{
    proc::{ExecutionState, Pid},
    sync::{DataLock, InteriorLock, InteriorLockMut, ReadDataLockGuard, ReadWriteDataLock},
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
    generation: AtomicUsize,
    pid_generator: AtomicUsize,
    // 用链表是因为可以单独上锁，修改的时候只需要锁前中两个元素就行
    head: Option<Arc<UnsafeCell<ProcessCell>>>,
    head_lock: ReadWriteSpinLock,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            generation: AtomicUsize::new(0),
            pid_generator: AtomicUsize::new(0),
            head: None,
            head_lock: ReadWriteSpinLock::new(),
        }
    }

    pub fn new_pid(&self) -> Pid {
        self.pid_generator.fetch_add(1, Ordering::Relaxed) as Pid
    }

    pub fn gen(&self) -> usize {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn add(&mut self, mut proc: Process) -> Pid {
        let pid = self.new_pid();
        let cell = ProcessCell::new(pid, proc, self.gen());
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

pub struct ThreadCell {
    inner: Thread,
    generation: AtomicUsize,
    next: Option<Arc<UnsafeCell<ThreadCell>>>,
    next_lock: ReadWriteSpinLock,
    // must acquire lock before modify the Thread data or put it in hart context
    state_lock: SpinLock,
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
    head: Option<Arc<UnsafeCell<ThreadCell>>>,
    head_lock: ReadWriteSpinLock,
    next: Option<Arc<UnsafeCell<ProcessCell>>>,
    next_lock: ReadWriteSpinLock,
}

impl ProcessCell {
    pub fn new(pid: Pid, proc: Process, initial_gen: usize) -> Self {
        let mut inner = proc;
        inner.pid = pid;
        let cell = ThreadCell::new();
        Self {
            inner,
            generation: AtomicUsize::new(initial_gen),
            head: Some(Arc::new(UnsafeCell::new(cell))),
            head_lock: ReadWriteSpinLock::new(),
            next: None,
            next_lock: ReadWriteSpinLock::new(),
        }
    }

    pub fn gen(&self) -> usize {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn add_gen(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct SmoothScheduler<T: Timer> {
    hartid: usize,
    timer: T,
    current: Option<(Arc<UnsafeCell<ProcessCell>>, Arc<UnsafeCell<ThreadCell>>)>,
}

impl<T: Timer> SmoothScheduler<T> {
    pub const fn new(id: usize, timer: T) -> Self {
        Self {
            hartid: id,
            timer,
            current: None,
        }
    }

    fn pick_process(&mut self) -> Arc<UnsafeCell<ProcessCell>> {
        loop {
            let table_generation = unsafe { PROC_TABLE.gen() };
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
                    if thread.inner.state == ExecutionState::Ready {
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
    fn add(&mut self, proc: Process) {
        unsafe { PROC_TABLE.add(proc) };
    }

    fn schedule(&mut self) {
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
                    //return (proc.inner.pid, thread.inner.tid);
                } else {
                    thread.inner.state = ExecutionState::Ready;
                    thread.state_lock.unlock();
                    let new_thread_ptr = self.pick_thread(proc);
                    let new_thread = unsafe { &mut *new_thread_ptr.get() };
                    new_thread.inner.state = ExecutionState::Running;
                    let pid = proc.inner.pid;
                    let tid = new_thread.inner.tid;
                    self.current = Some((current_process, new_thread_ptr));
                    //return (pid, tid);
                }
            } else {
                thread.inner.state = ExecutionState::Ready;
                thread.state_lock.unlock();
                let proc_ptr = self.pick_process();
                let proc = unsafe { &mut *proc_ptr.get() };
                let pid = proc.inner.pid;
                let new_thread_ptr = self.pick_thread(proc);
                let new_thread = unsafe { &mut *new_thread_ptr.get() };
                new_thread.inner.state = ExecutionState::Running;
                let tid = new_thread.inner.tid;
                self.current = Some((proc_ptr, new_thread_ptr));
                //return (pid, tid);
            }
        } else {
            let proc_ptr = self.pick_process();
            let proc = unsafe { &mut *proc_ptr.get() };
            let pid = proc.inner.pid;
            let new_thread_ptr = self.pick_thread(proc);
            let new_thread = unsafe { &mut *new_thread_ptr.get() };
            new_thread.inner.state = ExecutionState::Running;
            let tid = new_thread.inner.tid;
            self.current = Some((proc_ptr, new_thread_ptr));
            //return (pid, tid);
        }
    }

    fn next_timeslice(&self) -> usize {
        3
    }

    fn context(&self) -> (&Process, &Thread) {
        if let Some((process_ptr, thread_ptr)) = self.current.as_ref() {
            let process = unsafe { &mut *process_ptr.get() };
            let thread = unsafe { &mut *thread_ptr.get() };
            (&process.inner, &thread.inner)
        } else {
            panic!("no process selected")
        }
    }
}
