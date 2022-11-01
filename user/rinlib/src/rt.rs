use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use erhino_shared::process::{Signal, Termination};

use crate::call::{sys_exit, sys_extend, sys_signal_return};

#[allow(unused)]
const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

extern "C" {
    fn _segment_break();
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> = LockedHeapWithRescue::new(heap_rescue);

#[lang = "start"]
fn lang_start<T: Termination + 'static>(main: fn() -> T) -> ! {
    unsafe {
        sys_extend(_segment_break as usize, 4096, 0b011).unwrap();
        HEAP_ALLOCATOR.lock().init(_segment_break as usize, 4096);
    }
    let code = main().to_exit_code();
    unsafe {
        loop {
            sys_exit(code);
        }
    }
}

pub static mut SIGNAL_HANDLER: Option<fn(Signal)> = None;

// 这里有个问题就是 rinlib 会被动态链接，存在 rinlib 里的值会被共享吗？会的话那其实 HEAP_ALLOCATOR 也会。。不如不动态链接了。
pub fn signal_handler(signal: Signal) {
    if let Some(func) = unsafe { SIGNAL_HANDLER } {
        func(signal);
    }
    unsafe { sys_signal_return() };
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    todo!();
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    let single = 4096;
    let mut size = single;
    unsafe {
        while layout.size() > size {
            size *= 2;
        }
        let last = heap.stats_total_bytes() + _segment_break as usize;
        sys_extend(last, size, 0b011).unwrap();
        heap.add_to_heap(last, last + size);
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
