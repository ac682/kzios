use alloc::{string::String, vec::Vec};
use erhino_shared::{process::ProcessState, Pid};

use crate::trap::TrapFrame;

pub struct Process {
    name: String,
    pid: Pid,
    parent: Pid,
    trap: TrapFrame,
    state: ProcessState,
}

pub struct ProcessTable {
    inner: Vec<Process>,
    current: usize,
}
