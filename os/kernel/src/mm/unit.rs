use super::page::{PageTableEntry, PageTable, PageTableEntry32};

pub enum MemoryUnitError {
    EntryNotFound,
    RanOutOfFrames,
    EntryOverwrite,
    BufferOverflow,
}

pub struct MemoryUnit<E: PageTableEntry + Sized + 'static> {
    root: PageTable<E>,
}

impl<E: PageTableEntry + Sized + 'static> MemoryUnit<E> {
    pub fn new() -> Option<Self> {
        if let Some(table) = PageTable::<E>::new() {
            Some(Self { root: table })
        } else {
            None
        }
    }
    pub fn map() {}
}

pub fn init() {
    let table = MemoryUnit::<PageTableEntry32>::new().unwrap();
}
