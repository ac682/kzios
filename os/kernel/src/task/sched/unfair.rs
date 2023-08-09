use core::{
    cell::UnsafeCell,
    ops::DerefMut,
    panic,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
    task::Waker,
};

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use erhino_shared::{
    proc::Pid,
    sync::{DataLock, InteriorLock, ReadWriteDataLock},
};
use spin::Spin;

use crate::{
    sync::{
        spin::{ReadWriteSpinLock, SpinLock},
        up::UpSafeCell,
    },
    task::{proc::Process, thread::Thread},
    trap::TrapFrame,
};

use super::Scheduler;

type Locked<T> = ReadWriteDataLock<T, ReadWriteSpinLock>;
type Shared<T> = UpSafeCell<T>;

// timeslice in ticks
const QUANTUM: usize = 2;

static mut PROC_TABLE: ProcessTable = ProcessTable::new();

pub struct ProcessCell {
    // 只读数据，不需要锁
    inner: Process,
}

impl ProcessCell {
    pub fn new(proc: Process) -> Self {
        Self { inner: proc }
    }
}

pub struct ThreadCell {
    inner: Thread,
    proc: Arc<Locked<ProcessCell>>,
    generation: usize,
    next: Option<Arc<Shared<ThreadCell>>>,
    prev: Option<Weak<Shared<ThreadCell>>>,
    run_lock: SpinLock,
    ring_lock: SpinLock,
}

impl ThreadCell {
    pub fn new(inner: Thread, parent: Arc<Locked<ProcessCell>>, initial_gen: usize) -> Self {
        Self {
            inner,
            proc: parent,
            generation: initial_gen,
            next: None,
            prev: None,
            run_lock: SpinLock::new(),
            ring_lock: SpinLock::new(),
        }
    }

    pub fn check_and_set_if_behind(&mut self) -> bool {
        let table = unsafe { &PROC_TABLE };
        if table
            .generation
            .fetch_max(self.generation, Ordering::Relaxed)
            == self.generation
        {
            false
        } else {
            self.generation += 1;
            true
        }
    }
}

// 在调度开始时这个 head 必须有东西，因为“没有任何一个线程或所有线程全部失活时内核失去意义”
pub struct ProcessTable {
    raw: Locked<Vec<Arc<Locked<ProcessCell>>>>,
    generation: AtomicUsize,
    head: Option<Arc<Shared<ThreadCell>>>,
    head_lock: SpinLock,
    last: Option<Weak<Shared<ThreadCell>>>,
    last_lock: SpinLock,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            raw: Locked::new(Vec::new(), ReadWriteSpinLock::new()),
            generation: AtomicUsize::new(0),
            head: None,
            head_lock: SpinLock::new(),
            last: None,
            last_lock: SpinLock::new(),
        }
    }

    pub fn gen(&self) -> usize {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn add_process(&mut self, cell: ProcessCell) -> Arc<Locked<ProcessCell>> {
        let mut raw = self.raw.lock_mut();
        let sealed = Arc::new(Locked::new(cell, ReadWriteSpinLock::new()));
        let result = sealed.clone();
        raw.push(sealed);
        result
    }

    pub fn add_thread(&mut self, mut cell: ThreadCell) {
        self.last_lock.lock();
        if let Some(last) = &self.last {
            if let Some(upgrade) = last.upgrade() {
                upgrade.ring_lock.lock();
                let mutable = upgrade.get_mut();
                cell.prev = Some(last.clone());
                mutable.next = Some(Arc::new(Shared::new(cell)));
                upgrade.ring_lock.unlock();
            }
        } else {
            // head must be None, too
            self.head_lock.lock();
            let arc = Arc::new(Shared::new(cell));
            self.last = Some(Arc::downgrade(&arc));
            self.head = Some(arc);
            self.head_lock.unlock();
        }

        self.last_lock.unlock();
    }

    pub fn remove_thread(&mut self, cell: &mut ThreadCell) {
        if cell.next.is_none() {
            // it must be the last
            self.last_lock.lock();
            self.last = None;
            self.last_lock.unlock();
        }

        if let Some(prev) = &cell.prev {
            if let Some(upgrade) = prev.upgrade() {
                upgrade.ring_lock.lock();
                let mutable = upgrade.get_mut();
                mutable.next = cell.next.take();
                upgrade.ring_lock.unlock();
            }
        } else {
            // it must be the head
            self.head_lock.lock();
            self.head = cell.next.take();
            self.head_lock.unlock();
        }
    }

    pub fn move_next(
        &self,
        current: &Arc<Shared<ThreadCell>>,
        lock_acquire: bool,
    ) -> Arc<Shared<ThreadCell>> {
        if lock_acquire {
            current.ring_lock.lock();
        } else {
            if !current.ring_lock.try_lock() {
                return self.move_next(current, lock_acquire);
            }
        }
        if let Some(next) = &current.next {
            current.ring_lock.unlock();
            return next.clone();
        } else {
            self.head_lock.lock();
            current.ring_lock.unlock();
            if let Some(head) = &self.head {
                self.head_lock.unlock();
                return head.clone();
            } else {
                unreachable!("head can't be None while current exists");
            }
        }
    }

    pub fn move_next_until(
        &self,
        current: &Arc<Shared<ThreadCell>>,
        pred: fn(&Arc<Shared<ThreadCell>>) -> bool,
    ) -> Arc<Shared<ThreadCell>> {
        let mut one = current.clone();
        while !pred(&one) {
            one = self.move_next(&one, false);
        }
        one
    }

    pub fn move_next_from_head_until(
        &self,
        pred: fn(&Arc<Shared<ThreadCell>>) -> bool,
    ) -> Arc<Shared<ThreadCell>> {
        let mut result: Option<Arc<Shared<ThreadCell>>> = None;
        self.head_lock.lock();
        if let Some(head) = &self.head {
            result = Some(self.move_next_until(head, pred));
        }
        self.head_lock.unlock();
        result.expect("there must be one thread at least")
    }
}

pub struct UnfairScheduler {
    current: Option<Arc<Shared<ThreadCell>>>,
}

impl UnfairScheduler {
    pub const fn new() -> Self {
        Self { current: None }
    }

    fn find_next(&self) -> Arc<Shared<ThreadCell>> {
        let table = unsafe { &PROC_TABLE };
        let pred: fn(&Arc<Shared<ThreadCell>>) -> bool = |x| {
            let mutable = x.get_mut();
            mutable.run_lock.try_lock() && mutable.check_and_set_if_behind()
        };
        if let Some(current) = &self.current {
            table.move_next_until(current, pred)
        } else {
            table.move_next_from_head_until(pred)
        }
    }
}

impl Scheduler for UnfairScheduler {
    fn add(&mut self, proc: Process) {
        let mut table = unsafe { &mut PROC_TABLE };
        let thread = proc.spawn();
        let cell = ProcessCell::new(proc);
        let sealed = table.add_process(cell);
        let thread_cell = ThreadCell::new(thread, sealed, table.gen());
        table.add_thread(thread_cell);
    }

    fn schedule(&mut self) {
        // 采用 smooth 的代数算法，由于该算法存在进程间公平问题，干脆取消进程级别的公平比较，直接去保证线程公平，彻底放弃进程公平。
        if let Some(current) = &self.current {
            let mutable = current.get_mut();
            // ring unlocked, run locked
            if mutable.check_and_set_if_behind() {
                return;
            } else {
                mutable.run_lock.unlock();
            }
        }
        let next = self.find_next();
        self.current = Some(next.clone());
    }

    fn next_timeslice(&self) -> usize {
        QUANTUM
    }

    fn context(&self) -> (&Process, &Thread, &TrapFrame) {
        if let Some(current) = &self.current {
            (
                &unsafe { current.proc.access_unsafe() }.inner,
                &current.inner,
                &current.inner.frame,
            )
        } else {
            panic!("hart is not scheduling-prepared yet")
        }
    }
}
