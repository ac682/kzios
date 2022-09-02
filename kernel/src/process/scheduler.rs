use spin::Mutex;

use crate::Process;
use crate::process::scheduler::flat_scheduler::FlatScheduler;

mod flat_scheduler;

type SchedulerImpl = FlatScheduler;

lazy_static! {
    pub static ref SCHEDULER: Mutex<SchedulerImpl> = Mutex::new(SchedulerImpl::new());
}

trait ProcessScheduler {
    fn add_process(&mut self, proc: Process);
    fn exit_process(&mut self, exit_code: u32);
    // 进程只能在运行状态下退出
    fn switch_next(&mut self) -> usize;
    // 强制从一个进程切换到下一个进程,返回新进程 pid
    fn switch_to_user(&mut self);
    // 从当前进程开始跑,并且设定定时器
    fn timer_tick(&mut self);
    // 到达时间片,切换到下一个进程并设置定时器
    fn current(&mut self) -> Option<&mut Process>;
    fn len(&self) -> usize;
}

pub fn add_process(mut proc: Process) {
    let mut scheduler = SCHEDULER.lock();
    proc.pid = scheduler.len();
    scheduler.add_process(proc);
}

pub fn switch_to_user() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.switch_to_user();
}

pub fn forward_tick() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.timer_tick();
}

pub fn exit_process(exit_code: u32) {
    let mut scheduler = SCHEDULER.lock();
    scheduler.exit_process(exit_code);
}