use core::{alloc::Layout, hint::spin_loop, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use erhino_shared::proc::Termination;

use crate::{
    external::{_heap_start, _stack_start},
    sbi,
};

const HEAP_ORDER: usize = 64;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

const LOGO: &str = include_str!("../logo.txt");

#[lang = "start"]
fn rust_start<T: Termination + 'static>(main: fn() -> T, hartid: usize, _dtb_addr: usize) -> isize {
    // 流程：汇编中为进入 RUST 做准备，设置栈
    // rust_start 中 #0 核心做 RUST 环境准备，配置 alloc，其他核心等待
    //
    if hartid == 0 {
        // rust initialization
        unsafe {
            let heap_start = _heap_start as usize;
            let size = _stack_start as usize - heap_start;
            HEAP_ALLOCATOR.lock().init(heap_start, size);
        }
        rust_env_init();
        main();
    }
    panic!();
}

fn rust_env_init() {
    sbi::init();
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    // print!(
    //     "\x1b[0;31mKernel panicking at #{}: \x1b[0m",
    //     mhartid::read()
    // );
    // if let Some(location) = info.location() {
    //     println!(
    //         "file {}, {}: {}",
    //         location.file(),
    //         location.line(),
    //         info.message().unwrap()
    //     );
    // } else {
    //     println!("no information available.");
    // }
    loop {}
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    // let single = 4096;
    // let mut size = 1;
    // unsafe {
    //     while layout.size() > size * single {
    //         size *= 2;
    //     }
    //     if let Some(frame_start) = frame_alloc(size) {
    //         heap.add_to_heap(frame_start * single, (frame_start + size) * single);
    //     } else {
    //         panic!("kernel memory request but ran out of memory");
    //     }
    // }
}
