use erhino_shared::proc::{ExitCode, Pid, Tid};

use super::Process;

//pub mod flat;
pub mod smooth;

pub trait Scheduler {
    fn add(&self, proc: Process) -> Pid;
    fn tick(&mut self) -> (Pid, Tid);
    fn begin(&mut self);
    fn current(&mut self) -> Option<&mut Process>;
    fn find(&mut self, pid: Pid) -> Option<&Process>;
    fn find_mut(&mut self, pid: Pid) -> Option<&mut Process>;
    fn finish(&mut self, code: ExitCode);
    fn kill(&mut self, pid: Pid);
}
