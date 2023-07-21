use alloc::{borrow, vec::Vec};
use erhino_shared::mem::PageNumber;
use flagset::FlagSet;

use crate::{external::_memory_end};

use super::{
    frame::{self, FrameTracker},
    page::{PageTable, PageTableEntry, PageTableEntry32, PageTableEntryFlag},
};

#[derive(Debug)]
pub enum MemoryUnitError {
    EntryNotFound,
    RanOutOfFrames,
    EntryOverwrite,
    BufferOverflow,
}

pub struct MemoryUnit<E: PageTableEntry + Sized + 'static> {
    root: PageTable<E>,
    where_put_the_frame_tracker_of_root_for_recycling: FrameTracker
}

impl<E: PageTableEntry + Sized + 'static> MemoryUnit<E> {
    pub fn new() -> Result<Self, MemoryUnitError> {
        if let Some(frame) = frame::borrow(1) {
            Ok(Self {
                root: PageTable::<E>::from(frame.start()),
                where_put_the_frame_tracker_of_root_for_recycling: frame
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
        container: &mut PageTable<E>,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
        level: usize,
    ) -> Result<(), MemoryUnitError> {
        let start = Self::index_of_vpn(vpn, level);
        if level == 0 {
            // 只处理 count 中 0 级别的部分，未处理的应该再上轮递归（level = 1）中交给下次递归调用
            for i in 0..count {
                if container.create_leaf(start + i, ppn + i, flags).is_none() {
                    return Err(MemoryUnitError::EntryOverwrite);
                }
            }
            Ok(())
        } else {
            let end = Self::index_of_vpn(vpn + count, level);
            let diff = end - start;
            if diff > 1 {
                // 0:123:2 0:125:12 D 0:2:12
                // 2:45:12 2:47:24 D 0:2:12
                // 要求 vpn 和 ppn 都对 level 对齐
                todo!()
            } else if diff == 1 {
                todo!()
            } else {
                if let Some(table) = container.ensure_table_created(start, || frame::borrow(1)) {
                    Self::map_internal(table, vpn, ppn, count, flags, level - 1)
                } else {
                    Err(MemoryUnitError::RanOutOfFrames)
                }
            }
        }
    }

    fn index_of_vpn(vpn: PageNumber, level: usize) -> usize {
        // 9|9|9
        (vpn >> (level * E::SIZE)) & (1 << E::SIZE - 1)
    }

    fn is_page_number_aligned_to(number: PageNumber, level: usize) -> bool {
        number & (1 << (level * E::SIZE) - 1) == 0
    }
}

pub fn init() {
    let mut table = MemoryUnit::<PageTableEntry32>::new().unwrap();
    table
        .map(
            0x0,
            0x0,
            _memory_end as usize >> 12,
            PageTableEntryFlag::Valid
                | PageTableEntryFlag::Writeable
                | PageTableEntryFlag::Readable
                | PageTableEntryFlag::Executable,
        )
        .unwrap();
}
