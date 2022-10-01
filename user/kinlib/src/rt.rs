use core::{
    alloc::Layout,
    arch::global_asm,
};

use buddy_system_allocator::LockedHeap;

use crate::{process::Termination, syscall::sys_map};

global_asm!(include_str!("rt.asm"));

const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<HEAP_ORDER> = LockedHeap::empty();

extern "C" {
    fn _segment_break();
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    // TODO: mmap more
    panic!("Heap allocation error, layout = {:?}", layout);
}

// NOTE: 编译器会自己生成一个main, 该函数会调用 lang_start 并把用户的那个被混淆的 main 作为参数传入
#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    unsafe {
        // init heap
        // map some
        sys_map(_segment_break as u64 >> 12, 1, 0b110);
        HEAP_ALLOCATOR
            .lock()
            .init(_segment_break as usize, INITIAL_HEAP_SIZE);
    }

    // call main
    main().to_exit_code()
}
