use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use erhino_shared::{
    mem::Address,
    proc::{SystemSignal, Termination},
};

use crate::{
    call::{sys_exit, sys_extend, sys_signal_return},
    debug,
};

const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 32;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);
static mut SIGNAL_HANDLER: Option<fn(SystemSignal)> = None;

#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _: isize,
    _: *const *const u8,
    _: u8,
) -> isize {
    unsafe {
        let offset = sys_extend(INITIAL_HEAP_SIZE).expect("the first extend call failed");
        HEAP_ALLOCATOR
            .lock()
            .init(offset - INITIAL_HEAP_SIZE, INITIAL_HEAP_SIZE);
    }
    let code = main().to_exit_code();
    unsafe {
        loop {
            sys_exit(code).expect("this can't be wrong");
        }
    }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        debug!(
            "Panicking in {} at line {}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        debug!("Panicking: no information available.");
    }
    unsafe {
        loop {
            sys_exit(-1).expect("this can't be wrong");
        }
    }
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    debug!("rescue: ");
    let owned = heap.stats_total_bytes();
    let mut size = owned;
    while layout.size() > size {
        size *= 2;
    }
    unsafe {
        let call = sys_extend(size);
        match call {
            Ok(position) => heap.add_to_heap(position - size, position),
            Err(err) => panic!(
                "cannot request more memory region by extend sys call{:?}",
                err
            ),
        }
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub fn set_signal_handler(handler: fn(SystemSignal)) -> Address {
    unsafe {
        SIGNAL_HANDLER = Some(handler);
    }
    signal_handler as usize
}

fn signal_handler(signal: SystemSignal) {
    if let Some(handler) = unsafe { SIGNAL_HANDLER } {
        handler(signal)
    }
    unsafe {
        sys_signal_return().expect("wont failed if signal_handler called only by kernel");
    }
}
