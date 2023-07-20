use alloc::{borrow, vec::Vec};
use erhino_shared::mem::PageNumber;
use flagset::FlagSet;

use super::{
    frame::{self, FrameTracker},
    page::{PageTable, PageTableEntry, PageTableEntry32, PageTableEntryFlag},
};

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
    pub fn new() -> Result<Self, MemoryUnitError> {
        if let Some(frame) = frame::borrow(1) {
            Ok(Self {
                root: PageTable::<E>::from(frame),
            })
        } else {
            Err(MemoryUnitError::RanOutOfFrames)
        }
    }
    pub fn map<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        Self::map_internal(&mut self.root, vpn, ppn, count, flags, E::DEPTH - 1)
    }

    fn map_internal<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        parent: &mut PageTable<E>,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
        level: usize,
    ) -> Result<(), MemoryUnitError> {
        // vpn 分为 DEPTH 段，每段 SEGMENT 个 bit 大小
        let entry_count = PageTable::<E>::entry_count();
        // Sv39
        // map with no big page
        // 37:501:300(256)
        // ranging from S:37:501:300 to E:37:502:44(excluded) difference D:0:0:256
        // map with big page
        // 37:501:511(513)
        // ranging from S:37:501:511 to E:37:503:2(excluded), difference D:0:1:1
        // map with big pages but no big page
        // 37:501:1(600)
        // ranging from S:37:501:1 to E:37:502:89(excluded) difference D:0:1:88
        todo!()
    }

    fn index_of_vpn(vpn: PageNumber, level: usize) -> usize {
        let right = 12 + level * E::SIZE;
        if level == E::DEPTH - 1 {
            // 26|9|9|12 for Sv39 while DEPTH = 3, LENGTH = 56, SIZE = 9
            // selecting 56..30
            (vpn >> right) & (1 << (E::LENGTH - right) - 1)
        } else {
            (vpn >> right) & (1 << E::SIZE - 1)
        }
    }
}

pub fn init() {
    let table = MemoryUnit::<PageTableEntry32>::new();
}
