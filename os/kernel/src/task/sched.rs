use erhino_shared::proc::Pid;

use crate::trap::TrapFrame;

use super::proc::Process;

pub mod smooth;

pub trait Scheduler {
    fn add(&mut self, proc: Process) -> Pid;
    fn schedule(&mut self);
    fn context(&self) -> &TrapFrame;
}
