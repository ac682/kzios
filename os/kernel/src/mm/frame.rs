use buddy_system_allocator::LockedFrameAllocator;
use erhino_shared::mem::PageNumber;
use spin::Once;

use crate::external::{_kernel_end, _memory_end};

static mut FRAME_ALLOCATOR: Once<LockedFrameAllocator> = Once::new();

pub fn init() {
    let free_start: usize = _kernel_end as usize >> 12;
    let free_end = _memory_end as usize >> 12;
    unsafe {
        FRAME_ALLOCATOR.call_once(|| LockedFrameAllocator::new());
        FRAME_ALLOCATOR
            .get_mut()
            .unwrap()
            .lock()
            .add_frame(free_start, free_end);
    }
}

pub fn frame_alloc(count: usize) -> Option<PageNumber> {
    unsafe { FRAME_ALLOCATOR.get_mut().unwrap().lock().alloc(count) }
}

pub fn frame_dealloc(frame: PageNumber, count: usize) {
    unsafe {FRAME_ALLOCATOR.get_mut().unwrap().lock().dealloc((frame) as usize, count);
    }
}
