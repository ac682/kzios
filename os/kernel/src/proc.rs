pub(crate) mod pm;
pub(crate) mod sch;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use elf_rs::Elf;
use erhino_shared::{process::ProcessState, Address, Pid};

use crate::{mm::unit::MemoryUnit, trap::TrapFrame};

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
    pub fn from_bytes(data: &[u8]) -> Self {
        let elf = Elf::from_bytes(data);

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
