use buddy_system_allocator::LockedFrameAllocator;
use lazy_static::lazy_static;

use crate::external::{_kernel_end, _memory_end};

lazy_static! {
    static ref FRAME_ALLOCATOR: LockedFrameAllocator = LockedFrameAllocator::new();
}

pub fn init() {
    let free_start = _kernel_end as usize;
    let free_end = _memory_end as usize;
    FRAME_ALLOCATOR.lock().add_frame(free_start, free_end);
}
