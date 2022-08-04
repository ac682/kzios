pub mod address;
pub mod frame_allocator;
pub mod unit;
pub mod page_table;

use riscv::register::satp;
use riscv::register::satp::Mode;
use spin::Mutex;

use crate::mm::paged::{frame_allocator::FRAME_ALLOCATOR, unit::MemoryUnit};
use crate::paged::page_table::PageTableEntryFlags;

use self::{frame_allocator::FrameAllocator, page_table::PageTable};

extern "C" {
    fn _kernel_start();
    fn _kernel_end();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Mutex<MemoryUnit> = Mutex::new(MemoryUnit::new());
}

pub fn alloc() -> Option<usize> {
    FRAME_ALLOCATOR.lock().alloc()
}

pub fn free(ppn: usize) {
    FRAME_ALLOCATOR.lock().free(ppn)
}

pub fn init() {
    frame_allocator::init();
    let mut space = KERNEL_SPACE.lock();
    space.init(PageTable::new(2, alloc().unwrap()));space.map(0x1_0000, 0x1_0000, 1, PageTableEntryFlags::Readable | PageTableEntryFlags::Writeable | PageTableEntryFlags::Valid);
    space.map(
        _kernel_start as usize,
        _kernel_start as usize,
        (_kernel_end as usize - _kernel_start as usize) >> 12,
        PageTableEntryFlags::Executable | PageTableEntryFlags::Readable | PageTableEntryFlags::Writeable | PageTableEntryFlags::Valid,
    );
    //TODO: 这里还是M模式，分页不会生效！
    space.activate();
}
