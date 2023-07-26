use crate::trap::TrapFrame;

use super::proc::Process;

pub mod unfair;

pub trait Scheduler {
    fn add(&mut self, proc: Process);
    fn schedule(&mut self);
    fn context(&self) -> &TrapFrame;
}
