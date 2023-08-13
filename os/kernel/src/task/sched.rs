use erhino_shared::{mem::Address, proc::Pid};

use crate::{mm::ProcessAddressRegion, trap::TrapFrame};

use super::{proc::Process, thread::Thread};

pub mod unfair;

pub trait Scheduler {
    fn add(&mut self, proc: Process, parent: Option<Pid>) -> Pid;
    fn is_address_in(&self, addr: Address) -> Option<ProcessAddressRegion>;
    fn schedule(&mut self);
    fn next_timeslice(&self) -> usize;
    fn context(&self) -> Option<(Address, usize, Address)>;
    fn with_context<F: FnMut(&mut Process, &mut Thread, &mut TrapFrame)>(&self, func: F);
}
