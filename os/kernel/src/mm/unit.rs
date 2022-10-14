use core::f32::consts::E;

use erhino_shared::PageNumber;
use flagset::FlagSet;

use super::{
    frame::frame_alloc,
    page::{PageLevel, PageTable, PageTableEntryFlag, PageTableError},
};

pub struct MemoryUnit<'root> {
    root: PageTable<'root>,
}

impl<'root> MemoryUnit<'root> {
    pub fn new() -> Self {
        Self {
            root: PageTable::new(frame_alloc(1).unwrap() as u64, PageLevel::Giga),
        }
    }

    pub fn map<F: Into<FlagSet<PageTableEntryFlag>>>(
        &'root mut self,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), PageTableError> {
        // from vpn..vpn+count
        // from ppn..ppn+count
        let mut mapped = 0usize;
        // 从连续部分能作为一个 G-Page
        todo!()
    }

    pub fn unmap(&'root mut self, vpn: PageNumber) -> Result<(), PageTableError> {
        todo!();
    }
}
