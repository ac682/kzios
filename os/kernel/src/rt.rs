use alloc::{format, string::String};
use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use core::{alloc::Layout, arch::asm, panic::PanicInfo};
use erhino_shared::proc::Termination;
use riscv::register::{mhartid, misa};

use crate::{
    external::{
        _bss_end, _bss_start, _hart_num, _heap_start, _kernel_end, _memory_end, _memory_start,
        _stack_start,
    },
    mm::frame::frame_alloc,
    print, println,
};

const HEAP_ORDER: usize = 64;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

const LOGO: &str = include_str!("../logo.txt");

// only #0 goes here, others called in trap context
#[lang = "start"]
fn rust_start<T: Termination + 'static>(main: fn() -> T, _hartid: usize) -> isize {
    // boot stage #0: enter rust environment
    // boot stage #1: hart(core & trap context) initialization
    // 👆 both done in _start@assembly.asm
    // boot stage #2: board(memory & peripheral) initialization
    unsafe {
        let heap_start = _heap_start as usize;
        let size = _stack_start as usize - heap_start;
        HEAP_ALLOCATOR.lock().init(heap_start, size);

        // TODO: 暂时留着，以后板子通过其他方法发送信息
        board_init();
    }
    println!("{}\n\x1b[0;34mis still booting\x1b[0m", LOGO);
    print_isa();
    print_segments();
    // main() -> kernel_init() to setup peripheral -> kernel_main() to enter boot stage #3
    main();
    panic!("unreachable here");
    // do board clean
}

fn print_isa() {
    let isa = misa::read().unwrap();
    let xlen = isa.mxl();
    let mut isa_str = String::new();
    isa_str.push_str(&format!(
        "RV{}",
        match xlen {
            misa::MXL::XLEN32 => "32",
            misa::MXL::XLEN64 => "64",
            misa::MXL::XLEN128 => "128",
        }
    ));
    let bits = isa.bits() & 0x3FF_FFFF;
    for i in 0..26 {
        if (bits >> i) & 1 == 1 {
            isa_str.push((b'A' + i) as char);
        }
    }
    println!("ISA: {}", isa_str);
}

fn print_segments() {
    let stack_per_hart = (_kernel_end as usize - _stack_start as usize) / _hart_num as usize;
    println!(
        "memory@{:#x}..{:#x} {{",
        _memory_start as usize, _memory_end as usize
    );
    println!(
        "\tkernel@{:#x}..{:#x} {{",
        _memory_start as usize, _kernel_end as usize
    );
    println!(
        "\t\tbss@{:#x}..{:#x};",
        _bss_start as usize, _bss_end as usize
    );
    println!(
        "\t\tstack@{:#x}..{:#x} {{",
        _stack_start as usize, _kernel_end as usize
    );
    for i in 0..(_hart_num as usize) {
        println!(
            "\t\t\thart{}@{:#x}..{:#x};",
            i,
            _kernel_end as usize - stack_per_hart * (i + 1),
            _kernel_end as usize - stack_per_hart * i
        );
    }
    println!("\t\t}}");
    println!("\t}}");
    println!("}}");
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    let single = 4096;
    let mut size = 1;
    unsafe {
        while layout.size() > size * single {
            size *= 2;
        }
        if let Some(frame_start) = frame_alloc(size) {
            heap.add_to_heap(frame_start * single, (frame_start + size) * single);
        } else {
            panic!("kernel memory request but ran out of memory");
        }
    }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    print!(
        "\x1b[0;31mKernel panicking at #{}: \x1b[0m",
        mhartid::read()
    );
    if let Some(location) = info.location() {
        println!(
            "file {}, {}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    park();
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    // TODO: 移到内核外部空间之后有 Rescue 来扩充堆
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub fn park() -> ! {
    unsafe {
        loop {
            asm!("wfi");
        }
    }
}

extern "Rust" {
    #[linkage = "extern_weak"]
    pub fn board_init();
}
