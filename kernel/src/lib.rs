#![no_std]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]
#![allow(unused)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;

use core::arch::asm;
use core::slice::from_raw_parts;
use core::{arch::global_asm, panic};

use dtb_parser::device_tree::DeviceTree;
use dtb_parser::node::DeviceTreeNode;
use dtb_parser::prop::{NodeProperty, PropertyValue};
use spin::Mutex;

use mm::{
    heaped,
    paged::{self, alloc, page_table::PageTable},
};
use primitive::qemu;

use crate::board::BoardInfo;
use crate::external::{
    _kernel_end, _kernel_start, _memory_end, _memory_start, _stack_end, _stack_start,
};
use crate::lang_items::print;
use crate::paged::frame_allocator::{FrameAllocator, FRAME_ALLOCATOR};
use crate::process::scheduler::{add_process, switch_to_user};
use crate::process::Process;
use crate::timer::set_next_timer;

mod board;
mod external;
mod lang_items;
mod mm;
mod pmp;
mod primitive;
mod process;
mod syscall;
mod timer;
mod trap;
mod typedef;
mod utils;

global_asm!(include_str!("assembly.asm"));

#[no_mangle]
extern "C" fn kernel_main(
    hartid: usize,
    dtb_addr: usize,
    init0_addr: usize,
    init0_size: usize,
) -> ! {
    // kernel init
    pmp::init();
    mm::init();
    trap::init();
    // read device tree
    // 留以备用
    let info = parse_board_info(dtb_addr);
    // 日后换掉
    qemu::init();
    // ----- 初始化完成
    println!("Hello, World!");
    println!("hart id: #{}, device tree at: {:#x}", hartid, dtb_addr);
    let data = unsafe { from_raw_parts(init0_addr as *const u8, init0_size) };
    let process = Process::from_elf(data).unwrap();
    add_process(process);
    // ----- 进入用户空间, 此后内核仅在陷入中受理事件
    unsafe {
        // trap call, enter the userspace
        asm!("ecall", in("x10") 0);
        // unreachable
        loop {
            asm!("wfi")
        }
    }
}

fn parse_board_info(dtb_addr: usize) -> BoardInfo {
    // anything wrong just go panic
    if let Ok(tree) = DeviceTree::from_address(dtb_addr) {
        BoardInfo {}
    } else {
        panic!("Device tree cannot be reached");
    }
}
