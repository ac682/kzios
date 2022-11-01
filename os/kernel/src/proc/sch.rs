use erhino_shared::process::Pid;

use crate::timer::Timer;

use self::flat::FlatScheduler;

use super::Process;

pub mod flat;

pub trait Scheduler {
    fn add(proc: Process) -> Pid;
    fn tick(&mut self) -> Pid;
    fn begin(&mut self);
    fn current(&mut self) -> Option<&mut Process>;
    fn unlocked<F: Fn(&mut Process)>(&mut self, pid: Pid, func: F);
    fn finish(&mut self);
    fn kill(&mut self, pid: Pid);
}