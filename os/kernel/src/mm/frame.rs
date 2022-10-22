use buddy_system_allocator::{FrameAllocator, LockedFrameAllocator};
use erhino_shared::PageNumber;

use crate::{
    external::{_kernel_end, _memory_end},
    sync::{hart::HartLock, Lock},
};

static mut FRAME_ALLOCATOR: HartLock<FrameAllocator> = HartLock::empty();

pub fn init() {
    let free_start = _kernel_end as usize;
    let free_end = _memory_end as usize;
    unsafe {
        FRAME_ALLOCATOR.put(FrameAllocator::new());
        FRAME_ALLOCATOR.lock().add_frame(free_start, free_end);
    }
}

pub fn frame_alloc(count: usize) -> Option<PageNumber> {
    unsafe {
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator.alloc(count).map(|x| (x >> 12))
    }
}

pub fn frame_dealloc(frame: PageNumber, count: usize) {
    unsafe {
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator.dealloc((frame << 12) as usize, count)
    }
}
