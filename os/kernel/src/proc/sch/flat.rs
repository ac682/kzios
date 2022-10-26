use alloc::vec::Vec;
use erhino_shared::{process::ProcessState, Pid};
use riscv::register::{mhartid, mscratch};

use crate::{
    proc::Process,
    sync::{
        hart::{HartReadWriteLock, HartWriteLockGuard},
        Lock, ReadWriteLock,
    },
    timer,
    trap::TrapFrame,
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

    pub fn add(&mut self, proc: Process) {
        self.inner
            .push(HartReadWriteLock::new(ProcessCell::new(proc)));
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn next_availiable(&mut self) -> HartWriteLockGuard<ProcessCell> {
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

    pub fn next_quantum(&self) -> usize {
        let max = timer::time_to_cycles(50);
        let min = timer::time_to_cycles(10);
        let p = self.last_quantum as f32
            / (if self.out_time > self.in_time {
                (self.out_time - self.in_time)
            } else {
                1
            }) as f32;
        let i = -0.02 * p + 1.62;
        let n = (i * self.last_quantum as f32) as usize;
        if n > max {
            max
        } else if n < min {
            min
        } else {
            n
        }
    }
}
pub struct FlatScheduler {
    hartid: usize,
    current: Option<HartWriteLockGuard<'static, ProcessCell>>,
}

impl Scheduler for FlatScheduler {
    fn new(hartid: usize) -> Self {
        FlatScheduler {
            hartid,
            current: None,
        }
    }
    fn add(proc: Process) {
        let mut table = unsafe { PROC_TABLE.lock_mut() };
        table.add(proc);
    }
    fn tick(&mut self) {
        todo!("begin the first process");
    }
    fn begin(&mut self) {
        if unsafe { (*PROC_TABLE.access()).len() } > 0 {
            self.switch_next();
        } else {
            panic!("no process availiable");
        }
    }
}

impl FlatScheduler {
    fn switch_next(&mut self) -> Pid {
        unsafe {
            let mut table = PROC_TABLE.lock_mut();
            let mut process = unsafe {(*PROC_TABLE.access_mut()).next_availiable()};
            let next_pid = process.inner.pid;
            let time = timer::get_time();
            if let Some(current) = &mut self.current {
                if current.inner.state == ProcessState::Running {
                    current.inner.state = ProcessState::Ready;
                    current.out_time = time;
                }
            }
            mscratch::write(&process.inner.trap as *const TrapFrame as usize);
            process.inner.state = ProcessState::Running;
            process.in_time = time;
            let quantum = process.next_quantum();
            process.last_quantum = quantum;
            timer::set_timer(self.hartid, quantum, super::forawrd_tick);
            self.current = Some(process);
            next_pid
        }
    }
}
