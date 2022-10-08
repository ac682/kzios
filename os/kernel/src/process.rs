use alloc::{borrow::ToOwned, string::String, vec::Vec};
use erhino_shared::{process::ProcessState, Pid, Address};

use crate::{trap::TrapFrame, mm::unit::MemoryUnit};

pub struct Process<'root> {
    name: String,
    pid: Pid,
    parent: Pid,
    entry_point: Address,
    memory: MemoryUnit<'root>,
    trap: TrapFrame,
    state: ProcessState,
}

pub struct ProcessTable<'root> {
    inner: Vec<Process<'root>>,
    current: usize,
}

impl<'root> Process<'root> {
    pub fn from_bytes<F: Fn()>(data: &[u8]) -> Self {
        Self {
            name: "any".to_owned(),
            pid: 0,
            parent: 0,
            entry_point: 0,
            memory: MemoryUnit::new(),
            trap: TrapFrame::new(),
            state: ProcessState::Ready,
        }
    }
}

impl<'root> ProcessTable<'root> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            current: 0,
        }
    }
}
