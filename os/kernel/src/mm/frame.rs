use buddy_system_allocator::{FrameAllocator, LockedFrameAllocator};
use erhino_shared::mem::PageNumber;
use spin::Once;

use crate::{
    external::{_kernel_end, _memory_end}, sync::{DataLock, hart::HartLock, InteriorLock},
};

static mut FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();
static mut FRAME_LOCK: HartLock = HartLock::new();

pub fn init() {
    let free_start = _kernel_end as usize >> 12;
    let free_end = _memory_end as usize >> 12;
    unsafe {
        FRAME_ALLOCATOR.call_once(||FrameAllocator::new());
        FRAME_LOCK.lock();
        FRAME_ALLOCATOR.get_mut().unwrap().add_frame(free_start, free_end);
    }
}

pub fn frame_alloc(count: usize) -> Option<PageNumber> {
    unsafe {
        let lock = FRAME_LOCK.lock();
        let mut allocator = FRAME_ALLOCATOR.get_mut().unwrap();
        allocator.alloc(count)
    }
}

pub fn frame_dealloc(frame: PageNumber, count: usize) {
    unsafe {
        let lock = FRAME_LOCK.lock();
        let mut allocator = FRAME_ALLOCATOR.get_mut().unwrap();
        allocator.dealloc((frame) as usize, count)
    }
}
