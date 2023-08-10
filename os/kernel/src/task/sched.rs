use erhino_shared::{proc::Pid, mem::Address};

use crate::trap::TrapFrame;

use super::{proc::Process, thread::Thread};

pub mod unfair;

pub trait Scheduler {
    fn add(&mut self, proc: Process, parent: Option<Pid>) -> Pid;
    fn schedule(&mut self);
    fn next_timeslice(&self) -> usize;
    fn context(&self) -> (&Process, &Thread, Address);
}
