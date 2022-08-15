use riscv::register::satp;
use riscv::register::satp::Mode;
use spin::Mutex;

use crate::mm::paged::{frame_allocator::FRAME_ALLOCATOR, unit::MemoryUnit};
use crate::paged::page_table::PageTableEntryFlags;

use self::{frame_allocator::FrameAllocator, page_table::PageTable};

pub mod address;
pub mod frame_allocator;
pub mod page_table;
pub mod unit;

pub fn alloc() -> Option<usize> {
    FRAME_ALLOCATOR.lock().alloc()
}

pub fn free(ppn: usize) {
    FRAME_ALLOCATOR.lock().free(ppn)
}

pub fn init() {
    frame_allocator::init();
}
