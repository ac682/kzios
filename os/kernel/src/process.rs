use alloc::{borrow::ToOwned, string::String, vec::Vec};
use erhino_shared::{process::ProcessState, Pid, Address};

use crate::trap::TrapFrame;

pub struct Process {
    name: String,
    pid: Pid,
    parent: Pid,
    entry_point: Address,
    trap: TrapFrame,
    state: ProcessState,
}

pub struct ProcessTable {
    inner: Vec<Process>,
    current: usize,
}

impl Process {
    pub fn from_fn<F: Fn()>(func: F) -> Self {
        Self {
            name: "any".to_owned(),
            pid: 0,
            parent: 0,
            entry_point: &func as *const F as Address,
            trap: TrapFrame::new(),
            state: ProcessState::Ready,
        }
    }
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            current: 0,
        }
    }
}
