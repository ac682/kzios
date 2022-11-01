use core::cell::{RefCell, UnsafeCell};

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use erhino_shared::process::{Pid, ProcessState};
use riscv::register::{mhartid, mscratch};

use crate::{
    hart::{my_hart, Hart},
    proc::Process,
    sync::{
        cell::UniProcessCell,
        hart::{HartReadWriteLock, HartWriteLockGuard},
        optimistic::OptimisticLockGuard,
        Lock, ReadWriteLock,
    },
    timer::{self, hart::HartTimer, Timer},
    trap::TrapFrame, println,
};

use super::Scheduler;

static mut PROC_TABLE: HartReadWriteLock<ProcessTable> =
    HartReadWriteLock::new(ProcessTable::new());

struct ProcessTable {
    inner: Vec<HartReadWriteLock<ProcessCell>>,
    current: usize,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            current: 0,
        }
    }

    pub fn add(&mut self, mut proc: Process) -> Pid {
        let pid = self.inner.len();
        proc.pid = pid as Pid;
        self.inner
            .push(HartReadWriteLock::new(ProcessCell::new(proc)));
        pid as Pid
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn next_available(&mut self) -> HartWriteLockGuard<ProcessCell> {
        loop {
            let current = self.current;
            self.current = (self.current + 1) % self.inner.len();
            if !self.inner[current].is_locked()
                && unsafe { (*self.inner[current].access()).inner.state == ProcessState::Ready }
            {
                // 由于进程表只增不减少，应该释出一份 'static 的 Guard
                return self.inner[current].lock_mut();
            }
        }
    }
}
struct ProcessCell {
    inner: Process,
    // 单位都是 cycles
    in_time: usize,
    out_time: usize,
    last_quantum: usize,
}

impl ProcessCell {
    pub fn new(proc: Process) -> Self {
        Self {
            inner: proc,
            last_quantum: 50,
            in_time: 0,
            out_time: 0,
        }
    }
}
pub struct FlatScheduler<T: Timer + Sized> {
    hartid: usize,
    timer: Rc<RefCell<T>>,
    current: Option<HartWriteLockGuard<'static, ProcessCell>>,
}

impl<T: Timer + Sized> Scheduler for FlatScheduler<T> {
    fn add(proc: Process) -> Pid {
        let mut table = unsafe { PROC_TABLE.lock_mut() };
        table.add(proc)
    }
    fn tick(&mut self) -> Pid {
        self.switch_next()
    }
    fn begin(&mut self) {
        if unsafe { (*PROC_TABLE.access()).len() } > 0 {
            self.switch_next();
        } else {
            panic!("no process available");
        }
    }
    fn current(&mut self) -> Option<&mut Process> {
        if let Some(guard) = &mut self.current {
            Some(&mut guard.inner)
        } else {
            None
        }
    }

    fn finish(&mut self) {
        if let Some(process) = self.current() {
            if process.state == ProcessState::Running {
                // 进程调用 exit 之前会自己调用 wait 来等待子进程退出
                // TODO: 查找所有子进程，然后直接 kill with no mercy
                process.state = ProcessState::Dead;
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
}

impl<T: Timer + Sized> FlatScheduler<T> {
    pub fn new(hartid: usize, timer: Rc<RefCell<T>>) -> Self {
        FlatScheduler {
            hartid,
            timer,
            current: None,
        }
    }
    fn switch_next(&mut self) -> Pid {
        unsafe {
            let mut table = PROC_TABLE.lock_mut();
            let time = self.timer.borrow().get_cycles();
            if let Some(current) = &mut self.current {
                if current.inner.state == ProcessState::Running {
                    current.inner.state = ProcessState::Ready;
                    current.out_time = time;
                }
                self.current = None;
            }

            // 👆 换出之前的
            // 👇 换入新的

            let mut process = unsafe { (*PROC_TABLE.access_mut()).next_available() };
            let next_pid = process.inner.pid;
            process.inner.state = ProcessState::Running;
            process.in_time = time;
            let quantum = self.next_quantum(&process);
            process.last_quantum = quantum;
            let cycles = self.timer.borrow().ms_to_cycles(quantum);
            self.timer.borrow_mut().set_timer(cycles);
            self.current = Some(process);
            next_pid
        }
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
