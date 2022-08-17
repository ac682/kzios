#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]
#![allow(unused)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;

use core::{arch::global_asm, panic};
use core::arch::asm;
use core::borrow::Borrow;
use core::slice::from_raw_parts;

use fdt_rs::base::*;
use fdt_rs::common::item::UnwrappableDevTreeItem;
use fdt_rs::prelude::*;
use spin::Mutex;

use mm::{
    heaped,
    paged::{self, alloc, page_table::PageTable},
};
use primitive::qemu;

use crate::external::{_kernel_end, _kernel_start, _memory_end, _memory_start, _stack_end, _stack_start, _trap_stack_end, _trap_stack_start};
use crate::lang_items::print;
use crate::process::manager::add_process;
use crate::process::Process;
use crate::timer::set_next_timer;

mod lang_items;
mod mm;
mod primitive;
mod process;
mod syscall;
mod trap;
mod pmp;
mod external;
mod timer;
mod vfs;

global_asm!(include_str!("assembly.asm"));

#[repr(align(4))]
struct _Wrapper<T>(T);

pub const FDT: &[u8] = &_Wrapper(*include_bytes!("../platforms/qemu/device.dtb")).0;

#[no_mangle]
extern "C" fn main() -> ! {
    // kernel init
    //pmp::init();
    mm::init();
    trap::init();
    timer::init();
    vfs::init();
    // device init, this should be moved to init0 process
    qemu::init();
    // hello world
    println!("Hello, World!");
    //0x1d000, 0x2000
    // unsafe{
    //     let pointer = 0x87000000 as *const u8;
    //     let data = from_raw_parts(pointer,0x2000 );
    //     let dt = DevTree::new(data).unwrap();
    //     println!("{:?}",dt);
    // }
    let devtree = unsafe {

        // Get the actual size of the device tree after reading its header.
        let size = DevTree::read_totalsize(FDT).unwrap();
        let buf = &FDT[..size];

        // Create the device tree handle
        DevTree::new(buf).unwrap()
    };
    let mut node_iter = devtree.compatible_nodes("sifive,test1");
    while let Some(node) = node_iter.next().unwrap() {
        println!("{}", node.name().unwrap());
    }
    print_sections();
    // 进程有问题, 在切换时没有保存上一个进程的pc到结构体里
    // let process0 = Process::new_fn(init0);
    // add_process(process0);
    // set_next_timer();
    unsafe {
        loop {
            asm!("wfi")
        }
    }
}

#[no_mangle]
fn init0() {
    syscall(0, '0' as usize, 0, 0, 0);
    loop {}
}

fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) {
    let mut ret = 0usize;
    unsafe { asm!("ecall", inlateout("x10") arg0 => ret, in("x11") arg1, in("x12") arg2, in("x13") arg3, in("x17") id) };
}

fn print_sections() {
    let memory_start = _memory_start as usize;
    let kernel_start = _kernel_start as usize;
    let stack_start = _stack_start as usize;
    let stack_end = _stack_end as usize;
    let trap_stack_start = _trap_stack_start as usize;
    let trap_stack_end = _trap_stack_end as usize;
    let kernel_end = _kernel_end as usize;
    let memory_end = _memory_end as usize;

    println!("memory@{:#x}:{:#x}={}K {{", memory_start, memory_end, (memory_end - memory_start) / 1024);
    println!("  kernel@{:#x}:{:#x}={}K {{", kernel_start, kernel_end, (kernel_end - kernel_start) / 1024);
    println!("    stack@{:#x}:{:#x}={}K;", stack_start, stack_end, (stack_end - stack_start) / 1024);
    println!("    trap_stack@{:#x}:{:#x}={}K;", trap_stack_start, trap_stack_end, (trap_stack_end - trap_stack_start) / 1024);
    println!("  }}");
    println!("  user@{:#x}:{:#x}={}K;", kernel_end, memory_end, (memory_end - kernel_end) / 1024);
    println!("}}");
}

fn add_devices() {
    // uart
    // spi
    // gpio
}
