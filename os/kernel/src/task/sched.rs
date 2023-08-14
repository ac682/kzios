use erhino_shared::{
    mem::Address,
    proc::{Pid, Tid},
};

use crate::{mm::ProcessAddressRegion, trap::TrapFrame};

use super::{proc::Process, thread::Thread};

pub mod unfair;

pub trait ScheduleContext {
    fn pid(&self) -> Pid;
    fn tid(&self) -> Tid;
    fn process(&self) -> &mut Process;
    fn thread(&self) -> &mut Thread;
    fn trapframe(&self) -> &mut TrapFrame;
    fn add_proc(&self, proc: Process) -> Pid;
    fn add_thread(&self, thread: Thread) -> Tid;
}

pub trait Scheduler {
    type Context: ScheduleContext;
    fn add(&mut self, proc: Process, parent: Option<Pid>) -> Pid;
    fn is_address_in(&self, addr: Address) -> Option<ProcessAddressRegion>;
    fn schedule(&mut self);
    fn next_timeslice(&self) -> usize;
    fn context(&self) -> Option<(Address, usize, Address)>;
    fn with_context<F: FnMut(&Self::Context)>(&self, func: F);
}
