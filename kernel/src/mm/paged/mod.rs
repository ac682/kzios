pub mod frame_allocator;

use crate::{mm::paged::frame_allocator::FRAME_ALLOCATOR, println};

use self::frame_allocator::FrameAllocator;

pub fn alloc() -> Option<usize>{
    FRAME_ALLOCATOR.lock().alloc()
}

pub fn free(ppn: usize){
    FRAME_ALLOCATOR.lock().free(ppn)
}

pub fn init(){
    frame_allocator::init();
}