use buddy_system_allocator::FrameAllocator;

use crate::mm::paged::frame_allocator::frame_alloc;
use crate::trap::TrapFrame;
use crate::mm::paged::table::PageTable;

pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Dead
}

struct Process{
    trap_context: TrapFrame,
    stack_pointer: *mut u8,
    programmer_counter: usize,
    pid: usize,
    root: PageTable,
    state: ProcessState
}

impl Process{
    pub fn new() -> Self{
        todo!()
    }
}