use spin::{Mutex, MutexGuard};

use crate::Process;
use crate::process::scheduler::flat_scheduler::FlatScheduler;

use super::{Address, ExitCode, Pid};

mod flat_scheduler;

type SchedulerImpl = FlatScheduler;

lazy_static! {
    static ref SCHEDULER: Mutex<SchedulerImpl> = Mutex::new(SchedulerImpl::new());
}

pub trait ProcessScheduler {
    fn add_process(&mut self, proc: Process);
    // 进程只能在运行状态下退出
    fn exit_process(&mut self, exit_code: ExitCode);
    // 强制从一个进程切换到下一个进程,返回新进程 pid
    fn switch_next(&mut self) -> Pid;
    // 从当前进程开始跑,并且设定定时器
    fn switch_to_user(&mut self);
    // 到达时间片,切换到下一个进程并设置定时器
    fn timer_tick(&mut self);
    fn current(&mut self) -> Option<&mut Process>;
    fn len(&self) -> usize;
}

pub fn get_scheduler() -> MutexGuard<'static, SchedulerImpl> {
    if SCHEDULER.is_locked() {
        // TODO: finish kernel smp support and add a spin lock which carries the thread tag can safely unlock
        unsafe {
            SCHEDULER.force_unlock();
            SCHEDULER.lock()
        }
    } else {
        SCHEDULER.lock()
    }
}

pub fn add_process(mut proc: Process) -> Pid {
    let mut scheduler = get_scheduler();
    let pid = scheduler.len() as Pid;
    proc.pid = pid;
    scheduler.add_process(proc);
    pid
}

pub fn switch_to_user() {
    let mut scheduler = get_scheduler();
    scheduler.switch_to_user();
}

pub fn forward_tick() {
    let mut scheduler = get_scheduler();
    scheduler.timer_tick();
}

pub fn exit_process(exit_code: ExitCode) {
    let mut scheduler = get_scheduler();
    scheduler.exit_process(exit_code);
}

pub fn trap_with_current<F: Fn(&mut Process)>(func: F) {
    let mut scheduler = get_scheduler();
    if let Some(current) = scheduler.current() {
        func(current)
    }
}

#[deprecated]
pub fn move_to_next_instruction() {
    let mut scheduler = get_scheduler();
    if let Some(current) = scheduler.current() {
        current.move_to_next_instruction();
    }
}

#[deprecated]
pub fn read_process_byte(addr: u64) -> Result<u8, ()> {
    let mut scheduler = get_scheduler();
    if let Some(current) = scheduler.current() {
        current.memory.read_byte(addr)
    } else {
        Err(())
    }
}
