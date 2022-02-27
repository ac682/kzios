use alloc::vec::Vec;

use crate::config::MEMORY_END;
use crate::mm::paged::frame_allocator::{frame_alloc, FrameTracker, print_frame_use, frame_dealloc};

mod heap_allocator;
mod paged;

pub fn init() {
    heap_allocator::init_heap();
    
    //test();
}

#[allow(unused)]
pub fn test(){
    extern "C" {
        fn skernel();
        fn ekernel();
    }
    struct MemoryMap {
        kernel: (usize, usize),
        user: (usize, usize),
    }
    let map = MemoryMap {
        kernel: (skernel as usize, ekernel as usize - skernel as usize),
        user: (ekernel as usize, MEMORY_END - ekernel as usize),
    };
    println!("kernel starts at {:#x}, takes {:#x} bytes, \nuser space starts at {:#x}, {:#x} bytes available", map.kernel.0, map.kernel.1, map.user.0, map.user.1);

    let mut frames = Vec::<FrameTracker>::new();
    for i in 0..256{
        frames.push(frame_alloc().unwrap());
    }
    for i in (0..256).rev(){
        if i % 7 == 0{
            let frame = frames.remove(i);
            drop(frame)
        }
    }
    print_frame_use();
}
