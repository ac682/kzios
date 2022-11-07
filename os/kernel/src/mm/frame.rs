use buddy_system_allocator::FrameAllocator;
use erhino_shared::mem::PageNumber;
use spin::Once;

use crate::{
    external::{_kernel_end, _memory_end},
    sync::{hart::HartLock, InteriorLock},
};

static mut FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();
static mut FRAME_LOCK: HartLock = HartLock::new();

pub fn init() {
    let free_start = _kernel_end as usize >> 12;
    let free_end = _memory_end as usize >> 12;
    unsafe {
        FRAME_ALLOCATOR.call_once(|| FrameAllocator::new());
        FRAME_ALLOCATOR
            .get_mut()
            .unwrap()
            .add_frame(free_start, free_end);
    }
}

pub fn frame_alloc(count: usize) -> Option<PageNumber> {
    unsafe {
        FRAME_LOCK.lock();
        let allocator = FRAME_ALLOCATOR.get_mut().unwrap();
        let frame = allocator.alloc(count);
        FRAME_LOCK.unlock();
        frame
    }
}

pub fn frame_dealloc(frame: PageNumber, count: usize) {
    unsafe {
        FRAME_LOCK.lock();
        let allocator = FRAME_ALLOCATOR.get_mut().unwrap();
        allocator.dealloc((frame) as usize, count);
        FRAME_LOCK.unlock();
    }
}
