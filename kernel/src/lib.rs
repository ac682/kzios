#![no_std]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]
#![allow(unused)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;

use core::{arch::global_asm, panic};
use core::arch::asm;

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
use crate::process::Process;
use crate::process::scheduler::{add_process, switch_to_user};
use crate::timer::set_next_timer;

mod board;
mod external;
mod lang_items;
mod mm;
mod pmp;
mod primitive;
mod process;
mod system_call;
mod timer;
mod trap;

global_asm!(include_str!("assembly.asm"));

#[no_mangle]
extern "C" fn main(hartid: usize, dtb_addr: usize) -> ! {
    // kernel init
    pmp::init();
    mm::init();
    trap::init();
    // read device tree
    let info = parse_board_info(dtb_addr); // 留以备用
    qemu::init(); // 日后换掉
    // ----- 初始化完成
    println!("Hello, World!");
    println!("hart id: #{}, device tree at: {:#x}", hartid, dtb_addr);
    // print_sections();
    let data = include_bytes!("../../init0/target/riscv64gc-unknown-none-elf/debug/kzios_init0");
    add_process(Process::from_elf(data).unwrap());
    // ----- 进入用户空间, 此后内核仅在陷入中受理事件
    unsafe {
        asm!("ecall", in("x10") 0); // trap call, enter the userspace
    }
    // unreachable
    unsafe {
        loop {
            asm!("wfi")
        }
    }
}

fn init0() {
    loop {
        // syscall(0, '0' as usize, 0, 0, 0);
    }
}

fn init1() {
    syscall(0, '1' as usize, 0, 0, 0);
    syscall(0x22, 0, 0, 0, 0);
}

fn init2() {
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0, '2' as usize, 0, 0, 0);
    syscall(0x22, 0, 0, 0, 0);
}

fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) {
    let mut ret = 0usize;
    unsafe {
        asm!("ecall", inlateout("x10") arg0 => ret, in("x11") arg1, in("x12") arg2, in("x13") arg3, in("x17") id)
    };
}

fn print_sections() {
    let memory_start = _memory_start as usize;
    let kernel_start = _kernel_start as usize;
    let stack_start = _stack_start as usize;
    let stack_end = _stack_end as usize;
    let kernel_end = _kernel_end as usize;
    let memory_end = _memory_end as usize;

    println!(
        "memory@{:#x}:{:#x}={}K {{",
        memory_start,
        memory_end,
        (memory_end - memory_start) / 1024
    );
    println!(
        "  kernel@{:#x}:{:#x}={}K {{",
        kernel_start,
        kernel_end,
        (kernel_end - kernel_start) / 1024
    );
    println!(
        "    stack@{:#x}:{:#x}={}K;",
        stack_start,
        stack_end,
        (stack_end - stack_start) / 1024
    );
    println!("  }}");
    println!(
        "  user@{:#x}:{:#x}={}K;",
        kernel_end,
        memory_end,
        (memory_end - kernel_end) / 1024
    );
    println!("}}");
}

fn parse_board_info(dtb_addr: usize) -> BoardInfo {
    // anything wrong just go panic
    if let Ok(tree) = DeviceTree::from_address(dtb_addr) {
        BoardInfo {}
    } else {
        panic!("Device tree cannot be reached");
    }
}
