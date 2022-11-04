use core::cell::{RefCell, UnsafeCell};

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use erhino_shared::{
    mem::Address,
    proc::{Pid, ProcessState, WaitingReason},
};
use riscv::register::{mhartid, mscratch};

use crate::{
    hart::{my_hart, Hart},
    println,
    proc::Process,
    sync::{
        hart::{HartLock, HartReadWriteLock},
        DataLock, InteriorLock, InteriorReadWriteLock,
    },
    timer::{self, hart::HartTimer, Timer},
    trap::TrapFrame,
};

use super::Scheduler;

// 就不上锁了
static mut PROC_TABLE: ProcessTable = ProcessTable::new();

struct ProcessTable {
    lock: HartReadWriteLock,
    inner: Vec<ProcessCell>,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            lock: HartReadWriteLock::new(),
            inner: Vec::new(),
        }
    }

    pub fn lock(&mut self) {
        self.lock.lock();
    }

    pub fn lock_mut(&mut self) {
        self.lock.lock_mut();
    }

    pub fn unlock(&mut self) {
        self.lock.unlock();
    }
    pub fn add(&mut self, mut proc: Process) -> Pid {
        let pid = self.inner.len();
        proc.pid = pid as Pid;
        self.inner.push(ProcessCell::new(proc));
        pid as Pid
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn next_available(&mut self, cursor: usize) -> (usize, &mut ProcessCell) {
        let mut current = cursor;
        loop {
            current = (current + 1) % self.inner.len();
            if self.inner[current].lock.try_lock() {
                if self.inner[current].inner.state == ProcessState::Ready {
                    return (current, &mut self.inner[current]);
                } else {
                    self.inner[current].lock.unlock();
                }
            }
        }
    }
}

struct ProcessCell {
    lock: HartLock,
    inner: Process,
    // 单位都是 cycles
    in_time: usize,
    out_time: usize,
    last_quantum: usize,
}

impl ProcessCell {
    pub fn new(proc: Process) -> Self {
        Self {
            lock: HartLock::new(),
            inner: proc,
            last_quantum: 35,
            in_time: 0,
            out_time: 0,
        }
    }
}
pub struct FlatScheduler<T: Timer + Sized> {
    hartid: usize,
    current: usize,
    timer: Rc<RefCell<T>>,
    owned: Option<usize>,
}

impl<T: Timer + Sized> Scheduler for FlatScheduler<T> {
    fn add(&self, proc: Process) -> Pid {
        unsafe {
            PROC_TABLE.lock();
            let pid = PROC_TABLE.add(proc);
            PROC_TABLE.unlock();
            pid
        }
    }
    fn tick(&mut self) -> Pid {
        self.switch_next()
    }
    fn begin(&mut self) {
        if unsafe { PROC_TABLE.len() } > 0 {
            self.switch_next();
        } else {
            panic!("no process available");
        }
    }
    fn current(&mut self) -> Option<&mut Process> {
        if let Some(owned) = self.owned {
            if owned < unsafe { &PROC_TABLE }.len() {
                Some(&mut unsafe { &mut PROC_TABLE }.inner[owned].inner)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn finish(&mut self) {
        //let hartid = self.hartid;
        if let Some(process) = self.current() {
            if process.state == ProcessState::Running {
                // 进程调用 exit 之前会自己调用 wait 来等待子进程退出
                // TODO: 查找所有子进程，然后直接 kill with no mercy
                process.state = ProcessState::Dead;
                //println!("#{} Exit Pid={}", hartid, process.pid);
                // TODO: do process clean
            } else {
                // ??? 这个 finish 只能是运行中的程序转发，程序不在运行但是被调用，那就是出现了调度错误！
                panic!("mistakes must have be made before finish invoked");
            }
        }
        self.switch_next();
    }

    // 深度优先递归击杀子进程然后击杀自己
    fn kill(&mut self, pid: Pid) {
        todo!()
    }

    fn find(&mut self, pid: Pid) -> Option<&Process> {
        todo!()
    }

    fn find_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        unsafe {
            if (pid as usize) < PROC_TABLE.inner.len() {
                Some(&mut PROC_TABLE.inner[pid as usize].inner)
            } else {
                None
            }
        }
    }
}

impl<T: Timer + Sized> FlatScheduler<T> {
    pub fn new(hartid: usize, timer: Rc<RefCell<T>>) -> Self {
        FlatScheduler {
            hartid,
            current: 0,
            timer,
            owned: None,
        }
    }
    fn switch_next(&mut self) -> Pid {
        let time = self.timer.borrow().get_cycles();
        if let Some(owned) = self.owned.take() {
            let current = unsafe { &mut PROC_TABLE.inner[owned] };
            if current.inner.state == ProcessState::Running {
                current.inner.state = ProcessState::Ready;
            }
            current.out_time = time;
            current.lock.unlock();
        }

        // 👆 换出之前的
        // 👇 换入新的

        let (current, mut process) = unsafe { PROC_TABLE.next_available(self.current) };
        self.current = current;
        let next_pid = process.inner.pid;
        process.inner.state = ProcessState::Running;
        process.in_time = time;
        let quantum = self.next_quantum(&process);
        process.last_quantum = quantum;
        let cycles = self.timer.borrow().ms_to_cycles(quantum);
        // println!(
        //     "#{} -> {} @ {:#x} for slice_{} with {:#x}",
        //     self.hartid,
        //     next_pid,
        //     if process.inner.signal.pending > 0 {
        //         process.inner.signal.backup.pc
        //     } else {
        //         process.inner.trap.pc
        //     },
        //     quantum,
        //     process.inner.signal.pending
        // );
        if process.inner.has_signals_pending() {
            process.inner.enter_signal();
        }
        self.timer.borrow_mut().set_timer(cycles);
        self.owned = Some(current);
        next_pid
    }

    fn next_quantum(&self, proc: &ProcessCell) -> usize {
        let max = 50;
        let min = 10;
        let p = proc.last_quantum as i64
            / (if proc.out_time > proc.in_time {
                (proc.out_time - proc.in_time)
            } else {
                1
            }) as i64;
        let i = -2 * p + 162;
        let n = (i as usize * proc.last_quantum / 100);
        if n > max {
            max
        } else if n < min {
            min
        } else {
            n
        }
    }
}
