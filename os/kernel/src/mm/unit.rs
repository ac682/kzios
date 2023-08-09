use core::fmt::Display;

use erhino_shared::mem::PageNumber;
use flagset::FlagSet;
use spin::Once;

use crate::{
    external::{_kernel_start, _memory_end, _memory_start, _trampoline},
    mm::page::PageTableEntry39,
    println,
    trap::TrapFrame,
};

use super::{
    frame::{self, FrameTracker},
    page::{PageFlag, PageTable, PageTableEntry, PageTableEntry32},
};

type KernelUnit = MemoryUnit<PageTableEntry39>;

static mut KERNEL_UNIT: Once<KernelUnit> = Once::new();

#[derive(Debug)]
pub enum MemoryUnitError {
    EntryNotFound,
    RanOutOfFrames,
    EntryOverwrite,
    BufferOverflow,
}

pub struct MemoryUnit<E: PageTableEntry + Sized + 'static> {
    root: PageTable<E>,
    where_the_frame_tracker_of_root_for_recycling_put: FrameTracker,
}

impl<E: PageTableEntry + Sized + 'static> MemoryUnit<E> {
    pub fn new() -> Result<Self, MemoryUnitError> {
        if let Some(frame) = frame::borrow(1) {
            Ok(Self {
                root: PageTable::<E>::from(frame.start()),
                where_the_frame_tracker_of_root_for_recycling_put: frame,
            })
        } else {
            Err(MemoryUnitError::RanOutOfFrames)
        }
    }

    pub fn top_page_number() -> usize {
        1usize << ((E::DEPTH * E::SIZE + 12) - 1) >> 12
    }

    pub fn satp(&self) -> usize {
        let mode = 12 + E::SIZE * E::LENGTH;
        let mode_code = match mode {
            32 => 1,
            39 => 8,
            47 => 9,
            56 => 10,
            _ => 0,
        };
        (mode_code << 60)
            + self
                .where_the_frame_tracker_of_root_for_recycling_put
                .start()
    }

    pub fn map<F: Into<FlagSet<PageFlag>> + Copy>(
        &mut self,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        Self::map_internal(&mut self.root, vpn, ppn, count, flags, E::DEPTH - 1)
    }

    fn map_internal<F: Into<FlagSet<PageFlag>> + Copy>(
        container: &mut PageTable<E>,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
        level: usize,
    ) -> Result<(), MemoryUnitError> {
        // 标记 _:[N]:..
        // _ container 所在位置。递归不考虑外围循环，_ 之前可能也存在级别，但当前前轮递归无法考虑故省略
        // [N] 当前处理级别
        // .. 省略但存在可能多个级别
        let start = Self::index_of_vpn(vpn, level);
        let end = Self::index_of_vpn(vpn + count, level);
        if level == 0 {
            // ..:_:[123]/..:_:[223]
            // 不需要关心 count 是否存在溢出当前级别的问题。上级会处理好（且对 count 不溢出做检查和保证）。
            for i in 0..count {
                if container.create_leaf(start + i, ppn + i, flags).is_none() {
                    return Err(MemoryUnitError::EntryOverwrite);
                }
            }
            Ok(())
        } else {
            // ..:start:.. 到 ..:end:.. 可能跨越节也可能不跨越
            if end - start > 0 {
                // _:[2]:A:../_:[4]:B:..
                // 这里只取出第一段 [2] 并处理，剩下的段转发给下次同轮递归的这一分支
                if Self::is_page_number_aligned_to(vpn, level)
                    && Self::is_page_number_aligned_to(ppn, level)
                {
                    // _:[2]:00/_:[4]:B
                    // 超级页的包含的最小页数量
                    let size = PageTable::<E>::entry_count().pow(level as u32);
                    let mut remaining = count;
                    let mut round = 0usize;
                    let round_space = PageTable::<E>::entry_count() - start;
                    // 第一段可以成为超级页
                    while remaining >= size && round < round_space {
                        if container
                            .create_leaf(start + round, ppn + (round * size), flags)
                            .is_none()
                        {
                            return Err(MemoryUnitError::EntryOverwrite);
                        }
                        remaining -= size;
                        round += 1;
                    }

                    if remaining > 0 {
                        Self::map_internal(
                            container,
                            vpn + (size * round),
                            ppn + (size * round),
                            remaining,
                            flags,
                            level,
                        )
                    } else {
                        // 没有剩下！
                        Ok(())
                    }
                } else {
                    // _:[2]:3:../_:[4]:13:..
                    // [2] 成为表转发给次级
                    let size = PageTable::<E>::entry_count();
                    let remaining =
                        (size - Self::index_of_vpn(vpn, level - 1)) * size.pow(level as u32 - 1);
                    if let Some(table) = container.ensure_table_created(start, || frame::borrow(1))
                    {
                        Self::map_internal(table, vpn, ppn, remaining, flags, level - 1)?;
                        // vpn + remaining = _:[3]:00
                        // 虽然末尾都是0但ppn依旧无法对齐，不是超级页，且接下来都不是超级页
                        if count > remaining {
                            Self::map_internal(
                                table,
                                vpn + remaining,
                                ppn + remaining,
                                count - remaining,
                                flags,
                                level,
                            )
                        } else {
                            // 没有剩下！
                            Ok(())
                        }
                    } else {
                        Err(MemoryUnitError::RanOutOfFrames)
                    }
                }
            } else {
                // _:[1]:123/_:[1]:223
                // 对于不跨越的情况直接转发给 container[1].table() 处理
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
        (vpn >> (level * E::SIZE)) & ((1 << E::SIZE) - 1)
    }

    fn is_page_number_aligned_to(number: PageNumber, level: usize) -> bool {
        number & ((1 << (level * E::SIZE)) - 1) == 0
    }
}

impl<E: PageTableEntry + Sized + 'static> Display for MemoryUnit<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "Memory Mapping at {:#x}\n  Visual  -> Physical (  Length  )=0bDAGUXWRV",
            self.where_the_frame_tracker_of_root_for_recycling_put
                .start()
                << 12
        )?;
        let mut start_v = 0usize;
        let mut start_p = 0usize;
        let mut aggregated = 0usize;
        let mut bits = 0u64;
        let mut dirty = false;
        for (vpn, ppn, count, flags) in &self.root {
            let flag_bits = flags.bits();
            if start_v + aggregated == vpn && start_p + aggregated == ppn && flag_bits == bits {
                aggregated += count;
                dirty = true;
            } else {
                if (dirty) {
                    writeln!(
                        f,
                        "{:#010x}->{:#010x}({:#010x})={:#010b}",
                        start_v, start_p, aggregated, bits
                    )?;
                    dirty = false;
                }
                start_v = vpn;
                start_p = ppn;
                aggregated = count;
                bits = flag_bits;
                dirty = true;
            }
        }
        if dirty {
            writeln!(
                f,
                "{:#010x}->{:#010x}({:#010x})={:#010b}",
                start_v, start_p, aggregated, bits
            )?;
        }
        Ok(())
    }
}

pub fn init() {
    let memory_start = _memory_start as usize >> 12;
    let memory_end = _memory_end as usize >> 12;
    let mut unit = MemoryUnit::<PageTableEntry39>::new().unwrap();
    // mmio device space
    unit.map(
        0x0,
        0x0,
        memory_start,
        PageFlag::Valid | PageFlag::Writeable | PageFlag::Readable,
    )
    .unwrap();
    // sbi + kernel space
    unit.map(
        memory_start,
        memory_start,
        memory_end - memory_start,
        PageFlag::Valid | PageFlag::Writeable | PageFlag::Readable | PageFlag::Executable,
    )
    .unwrap();
    let top_number = KernelUnit::top_page_number();
    // trampoline code page
    unit.map(
        top_number,
        _trampoline as usize >> 12,
        1,
        PageFlag::Valid | PageFlag::Writeable | PageFlag::Readable | PageFlag::Executable,
    )
    .unwrap();
    // kernel has no trap frame so it has no trap frame mapped
    println!("{}", unit);
    unsafe {
        KERNEL_UNIT.call_once(|| unit);
    }
}
