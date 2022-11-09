use core::{alloc::Layout, arch::asm, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue, LockedHeap};
use erhino_shared::proc::{Signal, Termination};

use crate::{
    call::{sys_exit, sys_extend, sys_signal_return},
    dbg,
};

#[allow(unused)]
const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

extern "C" {
    fn _segment_break();
}

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> = LockedHeapWithRescue::new(heap_rescue);

#[lang = "start"]
fn lang_start<T: Termination + 'static>(main: fn() -> T) -> ! {
    unsafe {
        sys_extend(_segment_break as usize, 0x4000, 0b011);
        HEAP_ALLOCATOR.lock().init(_segment_break as usize, 0x4000);
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
fn handle_panic(info: &PanicInfo) -> ! {
    dbg!("Process panicking: ");
    if let Some(location) = info.location() {
        dbg!(
            "file {}, {}: {}\n",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        dbg!("no information available.\n");
    }
    loop {}
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    dbg!("rescue: ");
    let single = 0x4000;
    let mut size = single;
    while layout.size() > size {
        size *= 2;
    }
    let last = heap.stats_total_bytes() + _segment_break as usize;
    dbg!("{:#x}..{:#x}", last, last + size);
    unsafe {
        sys_extend(last, size, 0b011);
        heap.add_to_heap(last, last + size);
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
