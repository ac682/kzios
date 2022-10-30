use crate::timer::Timer;

use self::flat::FlatScheduler;

use super::Process;

pub mod flat;

pub trait Scheduler {
    fn add(proc: Process);
    fn tick(&mut self);
    fn begin(&mut self);
    fn current(&mut self) -> Option<&mut Process>;
}