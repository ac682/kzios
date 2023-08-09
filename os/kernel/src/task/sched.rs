use erhino_shared::proc::Pid;

use crate::trap::TrapFrame;

use super::{proc::Process, thread::Thread};

pub mod smooth;
pub mod unfair;

pub trait Scheduler {
    fn add(&mut self, proc: Process);
    fn schedule(&mut self);
    fn next_timeslice(&self) -> usize;
    fn context(&self) -> (&Process, &Thread);
}
