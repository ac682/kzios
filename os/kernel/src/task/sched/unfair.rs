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
use elf_rs::ElfHeader;
use erhino_shared::{
    mem::{Address, PageNumber},
    proc::{ExecutionState, Pid, Tid},
    sync::{DataLock, InteriorLock, ReadWriteDataLock},
};
use flagset::FlagSet;
use spin::Spin;

use crate::{
    external::_user_trap,
    mm::{
        page::{PageEntryFlag, PageEntryImpl, PageTableEntry, PAGE_SIZE},
        unit::{AddressSpace, MemoryUnit},
        ProcessAddressRegion,
    },
    println,
    sync::{
        spin::{ReadWriteSpinLock, SpinLock},
        up::UpSafeCell,
    },
    task::{proc::Process, thread::Thread},
    trap::TrapFrame,
};

use super::Scheduler;

// 日后替换成 InterruptSafeLock
type Locked<T> = DataLock<T, SpinLock>;
type Shared<T> = UpSafeCell<T>;

// timeslice in ticks
const QUANTUM: usize = 2;

static mut PROC_TABLE: ProcessTable = ProcessTable::new();

const THREAD_STACK_SIZE: usize = 8 * 1024 * 1024;

struct ProcessLayout {
    // 跳板地址向上是跳板页，向下是 TrapFrame
    trampoline: Address,
    stack: Address,
    heap: Address,
    thread_count: usize,
}

impl ProcessLayout {
    pub fn new(trampoline: Address, stack: Address, heap: Address) -> Self {
        Self {
            trampoline: trampoline,
            stack: stack,
            heap: heap,
            thread_count: 1,
        }
    }

    pub fn is_address_in(&self, addr: Address) -> ProcessAddressRegion {
        match MemoryUnit::<PageEntryImpl>::is_address_in(addr) {
            AddressSpace::Invalid => ProcessAddressRegion::Invalid,
            AddressSpace::Kernel => {
                let count = self.thread_count;
                let diff = (self.trampoline - addr) / TRAPFRAME_SIZE;
                ProcessAddressRegion::TrapFrame(diff as Tid)
            }
            AddressSpace::User => {
                if addr < self.heap {
                    ProcessAddressRegion::Program
                } else {
                    let diff = (self.stack - addr - 1) / THREAD_STACK_SIZE;
                    let count = self.thread_count;
                    if diff < count {
                        ProcessAddressRegion::Stack(diff as Tid)
                    } else {
                        ProcessAddressRegion::Heap
                    }
                }
            }
        }
    }
}

// 只有 next, prev 需要用 ring_lock, head 用 head_lock, inner 和 layout 则需要手动获得中断安全锁
struct ProcessCell {
    inner: Process,
    id: Pid,
    parent: Pid,
    layout: ProcessLayout,
    head: Arc<Shared<ThreadCell>>,
    head_lock: SpinLock,
    next: Option<Arc<Shared<ProcessCell>>>,
    prev: Option<Weak<Shared<ProcessCell>>>,
    ring_lock: SpinLock,
    state_lock: SpinLock,
}

const TRAPFRAME_SIZE: usize = 1024;
const TRAPFRAME_HOLD: usize = PAGE_SIZE / TRAPFRAME_SIZE;

impl ProcessCell {
    pub fn new(
        proc: Process,
        pid: Pid,
        parent: Pid,
        main: ThreadCell,
        layout: ProcessLayout,
    ) -> Self {
        let mut mutable = proc;
        mutable.memory.map(
            layout.trampoline >> 12,
            _user_trap as usize >> 12,
            1,
            PageEntryFlag::PrefabUserTrampoline,
        );
        Self {
            inner: mutable,
            id: pid,
            parent: parent,
            layout: layout,
            head: Arc::new(Shared::new(main)),
            head_lock: SpinLock::new(),
            next: None,
            prev: None,
            ring_lock: SpinLock::new(),
            state_lock: SpinLock::new(),
        }
    }

    pub fn address_of_trapframe<E: PageTableEntry>(trampoline: Address, id: Tid) -> Address {
        // 最高页留给跳板，倒数第二个开始向下分配
        let start_page_number = (trampoline >> 12) - 1;
        let block = id as usize / TRAPFRAME_HOLD;
        let index = id as usize % TRAPFRAME_HOLD;
        ((start_page_number - block) << 12) + index * TRAPFRAME_SIZE
    }

    pub fn address_of_stack(stack: Address, id: Tid) -> Address {
        stack - (id as usize * THREAD_STACK_SIZE) - 1
    }

    pub fn move_next(
        &self,
        current: &Arc<Shared<ThreadCell>>,
        lock_acquire: bool,
    ) -> Option<Arc<Shared<ThreadCell>>> {
        if lock_acquire {
            current.ring_lock.lock();
        } else {
            if !current.ring_lock.try_lock() {
                return self.move_next(current, lock_acquire);
            }
        }
        if let Some(next) = &current.next {
            current.ring_lock.unlock();
            Some(next.clone())
        } else {
            None
        }
    }

    pub fn count_if_match_until(&self, pred: fn(&Arc<Shared<ThreadCell>>) -> bool) -> usize {
        self.head_lock.lock();
        let mut count = 0usize;
        let mut one = self.head.clone();
        while pred(&one) {
            count += 1;
            if let Some(next) = self.move_next(&one, true) {
                one = next;
            } else {
                break;
            }
        }
        self.head_lock.unlock();
        count
    }

    fn find_gap(&self) -> Tid {
        self.head_lock.lock();
        let mut current = self.head.clone();
        let mut tid = current.id + 1;
        while let Some(next) = self.move_next(&current, true) {
            if tid == next.id {
                tid = next.id + 1;
                current = next;
            } else {
                break;
            }
        }
        tid
    }

    pub fn new_tid(&self) -> Tid {
        self.find_gap()
    }

    pub fn ensure_page_created<F: Into<FlagSet<PageEntryFlag>> + Copy>(
        &mut self,
        number: PageNumber,
        flags: F,
    ) -> bool {
        self.state_lock.lock();
        if !self.inner.memory.is_page_created(number) {
            self.inner.memory.fill(number, 1, flags);
            self.state_lock.unlock();
            true
        } else {
            self.state_lock.unlock();
            false
        }
    }

    pub fn struct_at<T: Sized>(&self, addr: Address) -> &mut T {
        self.state_lock.lock();
        let (physical, _) = self
            .inner
            .memory
            .translate(addr)
            .expect("the page struct at has not been created");
        self.state_lock.unlock();
        unsafe { unsafe { &mut *(physical as *mut T) } }
    }
}

struct ThreadCell {
    inner: Thread,
    id: Tid,
    proc: Weak<Shared<ProcessCell>>,
    generation: usize,
    trapframe: Address,
    next: Option<Arc<Shared<ThreadCell>>>,
    run_lock: SpinLock,
    ring_lock: SpinLock,
}

impl ThreadCell {
    pub fn new(inner: Thread, tid: Tid, initial_gen: usize, trapframe: Address) -> Self {
        Self {
            inner,
            id: tid,
            proc: Weak::new(),
            generation: initial_gen,
            trapframe,
            next: None,
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
            self.generation += 1;
            true
        } else {
            false
        }
    }
}

// 在调度开始时这个 head 必须有东西，因为“没有任何一个线程或所有线程全部失活时内核失去意义”
struct ProcessTable {
    generation: AtomicUsize,
    pid_generator: AtomicUsize,
    head: Option<Arc<Shared<ProcessCell>>>,
    head_lock: SpinLock,
    last: Option<Weak<Shared<ProcessCell>>>,
    last_lock: SpinLock,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            generation: AtomicUsize::new(0),
            pid_generator: AtomicUsize::new(0),
            head: None,
            head_lock: SpinLock::new(),
            last: None,
            last_lock: SpinLock::new(),
        }
    }

    pub fn gen(&self) -> usize {
        self.generation.load(Ordering::Relaxed)
    }

    pub fn add_process(&mut self, mut cell: ProcessCell) -> Arc<Shared<ProcessCell>> {
        self.last_lock.lock();
        if let Some(last) = &self.last {
            let upgrade = last.upgrade().expect("last exists but pointers to null");
            upgrade.ring_lock.lock();
            let mut mutable = upgrade.get_mut();
            cell.prev = Some(last.clone());
            let sealed = Arc::new(Shared::new(cell));
            mutable.next = Some(sealed.clone());
            self.last = Some(Arc::downgrade(&sealed));
            self.last_lock.unlock();
            return sealed;
        } else {
            // head & last both None
            self.head_lock.lock();
            let sealed = Arc::new(Shared::new(cell));
            self.last = Some(Arc::downgrade(&sealed));
            self.head = Some(sealed.clone());
            self.last_lock.unlock();
            self.head_lock.unlock();
            return sealed;
        }
    }

    pub fn move_next_process(
        &self,
        current: &Arc<Shared<ProcessCell>>,
        repeat: bool,
    ) -> Option<Arc<Shared<ProcessCell>>> {
        current.ring_lock.lock();
        if let Some(next) = &current.next {
            current.ring_lock.unlock();
            Some(next.clone())
        } else {
            current.ring_lock.unlock();
            if repeat {
                self.head_lock.lock();
                if let Some(head) = &self.head {
                    self.head_lock.unlock();
                    Some(head.clone())
                } else {
                    self.head_lock.unlock();
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn move_next_thread(
        &self,
        proc: &Arc<Shared<ProcessCell>>,
        thread: &Arc<Shared<ThreadCell>>,
        repeat: bool,
    ) -> Option<(Arc<Shared<ProcessCell>>, Arc<Shared<ThreadCell>>)> {
        thread.ring_lock.lock();
        if let Some(next_thread) = &thread.next {
            thread.ring_lock.unlock();
            Some((proc.clone(), next_thread.clone()))
        } else {
            thread.ring_lock.unlock();
            proc.ring_lock.lock();
            if let Some(next_proc) = &proc.next {
                proc.ring_lock.unlock();
                next_proc.head_lock.lock();
                let next_thread = next_proc.head.clone();
                next_proc.head_lock.unlock();
                Some((next_proc.clone(), next_thread))
            } else {
                proc.ring_lock.unlock();
                if repeat {
                    self.head_lock.lock();
                    if let Some(head) = &self.head {
                        self.head_lock.unlock();
                        head.head_lock.lock();
                        let next_thread = head.head.clone();
                        head.head_lock.unlock();
                        Some((head.clone(), next_thread))
                    } else {
                        self.head_lock.unlock();
                        unreachable!("head cannot be None in the whole scheduler life-cycle")
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn move_next_thread_until(
        &self,
        proc: &Arc<Shared<ProcessCell>>,
        thread: &Arc<Shared<ThreadCell>>,
        pred: fn(&Arc<Shared<ProcessCell>>, &Arc<Shared<ThreadCell>>) -> bool,
        repeat: bool,
    ) -> Option<(Arc<Shared<ProcessCell>>, Arc<Shared<ThreadCell>>)> {
        let mut one_proc = proc.clone();
        let mut one_thread = thread.clone();
        while !pred(&one_proc, &one_thread) {
            if let Some((next_proc, next_thread)) =
                self.move_next_thread(&one_proc, &one_thread, repeat)
            {
                one_proc = next_proc;
                one_thread = next_thread;
            } else {
                return None;
            }
        }
        Some((one_proc, one_thread))
    }
}

pub struct UnfairScheduler {
    hartid: usize,
    current: Option<(Arc<Shared<ProcessCell>>, Arc<Shared<ThreadCell>>)>,
}

impl UnfairScheduler {
    pub const fn new(hartid: usize) -> Self {
        Self {
            hartid,
            current: None,
        }
    }

    fn new_pid(&self) -> Pid {
        let table = unsafe { &PROC_TABLE };
        table.pid_generator.fetch_add(1, Ordering::Relaxed) as Pid
    }

    fn find_next(&self) -> Option<(Arc<Shared<ProcessCell>>, Arc<Shared<ThreadCell>>)> {
        let table = unsafe { &PROC_TABLE };
        let pred: fn(&Arc<Shared<ProcessCell>>, &Arc<Shared<ThreadCell>>) -> bool = |p, t| {
            if t.inner.state == ExecutionState::Ready && t.run_lock.try_lock() {
                let mutable = t.get_mut();
                if mutable.check_and_set_if_behind() {
                    mutable.inner.state = ExecutionState::Running;
                    true
                } else {
                    t.run_lock.unlock();
                    false
                }
            } else {
                false
            }
        };
        if let Some((p, t)) = &self.current {
            table.move_next_thread_until(p, t, pred, true)
        } else {
            table.head_lock.lock();
            if let Some(process) = &table.head {
                table.head_lock.unlock();
                process.head_lock.lock();
                let thread = &process.head;
                process.head_lock.unlock();
                table.move_next_thread_until(process, thread, pred, true)
            } else {
                panic!("head can not be None during scheduler life-cycle")
            }
        }
    }
}

impl Scheduler for UnfairScheduler {
    fn add(&mut self, proc: Process, parent: Option<Pid>) -> Pid {
        let mut table = unsafe { &mut PROC_TABLE };
        let pid = self.new_pid();
        let parent_id = if let Some(parent) = parent {
            parent
        } else {
            pid
        };
        let layout = ProcessLayout::new(
            PageEntryImpl::top_address() & !0xFFF,
            PageEntryImpl::space_size(),
            0,
        );
        let entry_point = proc.entry_point;
        let trampoline_address = layout.trampoline;
        let stack_address = ProcessCell::address_of_stack(layout.stack, 0);
        let trapframe_address =
            ProcessCell::address_of_trapframe::<PageEntryImpl>(layout.trampoline, 0);
        let thread = ThreadCell::new(Thread::new("main"), 0, table.gen(), trapframe_address);
        let mut cell = ProcessCell::new(proc, pid, parent_id, thread, layout);
        cell.ensure_page_created(trapframe_address >> 12, PageEntryFlag::PrefabUserTrapframe);
        let trapframe = cell.struct_at::<TrapFrame>(trapframe_address);
        trapframe.init(self.hartid, entry_point, stack_address, trampoline_address);
        let sealed = table.add_process(cell);
        sealed.head_lock.lock();
        let mutable = sealed.head.get_mut();
        mutable.proc = Arc::downgrade(&sealed);
        sealed.head_lock.unlock();
        pid
    }

    fn is_address_in(&self, addr: Address) -> ProcessAddressRegion {
        if let Some((p, _)) = &self.current {
            p.state_lock.lock();
            let result = p.layout.is_address_in(addr);
            p.state_lock.unlock();
            result
        } else {
            unreachable!(
                "previous process want to see the address region while it is not in current field"
            )
        }
    }

    fn schedule(&mut self) {
        // 采用 smooth 的代数算法，由于该算法存在进程间公平问题，干脆取消进程级别的公平比较，直接去保证线程公平，彻底放弃进程公平。
        if let Some((p, t)) = &self.current {
            let mutable = t.get_mut();
            // ring unlocked, run locked
            if mutable.check_and_set_if_behind() {
                return;
            } else {
                mutable.run_lock.unlock();
                mutable.inner.state = ExecutionState::Ready;
            }
        }
        let next = self.find_next();
        self.current = next;
    }

    fn next_timeslice(&self) -> usize {
        QUANTUM
    }

    fn context(&self) -> (Address, usize, Address) {
        if let Some((p, t)) = &self.current {
            let satp = p.inner.memory.satp();
            (p.layout.trampoline, satp, t.trapframe)
        } else {
            // go IDLE
            panic!("hart is not scheduling-prepared yet")
        }
    }

    fn with_context<F: Fn(&mut Process, &mut Thread)>(&self, func: F) {
        if let Some((p, t)) = &self.current {
            p.state_lock.lock();
            func(&mut p.get_mut().inner, &mut t.get_mut().inner);
            p.state_lock.unlock();
        } else {
            // go IDLE
            panic!("hart is not scheduling-prepared yet")
        }
    }
}
