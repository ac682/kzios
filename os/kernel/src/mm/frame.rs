use buddy_system_allocator::{FrameAllocator, LockedFrameAllocator};

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

pub fn frame_alloc(count: usize) -> Option<usize> {
    unsafe {
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator.alloc(count)
    }
}

pub fn frame_dealloc(frame: usize, count: usize) {
    unsafe {
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator.dealloc(frame, count)
    }
}
