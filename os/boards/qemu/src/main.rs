#![no_std]

extern crate alloc;
extern crate erhino_kernel;

use core::fmt::{Arguments, Result, Write};

use alloc::borrow::ToOwned;
use dtb_parser::{prop::PropertyValue, traits::HasNamedProperty};
use erhino_kernel::{board::BoardInfo, env, kernel_init, kernel_main, proc::{Process}, sync::{hart::HartLock, InteriorLock}};
use tar_no_std::TarArchiveRef;

pub use erhino_kernel::prelude::*;

// 测试用，日后 initfs 应该由 board crate 提供
// board crate 会在 artifacts 里选择部分包括驱动添加到 initfs 里
const INITFS: &[u8] = include_bytes!("../../../../artifacts/initfs.tar");

fn main() {
    // prepare BoardInfo
    let dtb_addr = env::args()[1] as usize;
    let tree = dtb_parser::device_tree::DeviceTree::from_address(dtb_addr).unwrap();
    // println!("{}", tree);
    let mut clint_base = 0usize;
    let mut timebase_frequency = 0usize;
    for node in tree.into_iter() {
        if node.name().starts_with("clint") {
            if let Some(prop) = node.find_prop("reg") {
                if let PropertyValue::Address(address, _size) = prop.value() {
                    clint_base = *address as usize;
                }
            }
        }else if node.name() == "cpus"{
            if let Some(prop) = node.find_prop("timebase-frequency"){
                if let PropertyValue::Integer(frequency) = prop.value(){
                    timebase_frequency = *frequency as usize;
                }
            }
        }
    }
    let info = BoardInfo {
        name: "qemu".to_owned(),
        base_frequency: timebase_frequency,
        mswi_address: clint_base,
        mtimer_address: clint_base + 0x4000,
        mtime_address: clint_base + 0xbff8,
    };
    kernel_init(info);
    // add processes to scheduler
    let archive = TarArchiveRef::new(INITFS);
    let user_init = archive
        .entries()
        .find(|f| f.filename().as_str() == "user_init")
        .unwrap();
    let systems = archive.entries().filter(|f| f.filename().starts_with("system"));
    for system in systems{
        let process = Process::from_elf(system.data(), system.filename().as_str()).unwrap();
        add_flat_process(process);
    }
    let user_init_proc = Process::from_elf(user_init.data(), user_init.filename().as_str()).unwrap();
    //add_flat_process(user_init_proc);
    kernel_main();
}

#[export_name = "board_write"]
pub fn uart_write(args: Arguments) {
    NS16550a.write_fmt(args).unwrap();
}

#[export_name = "board_init"]
pub fn board_init() {
    ns16550a_init();
}

pub struct NS16550a;

impl Write for NS16550a {
    fn write_str(&mut self, s: &str) -> Result {
        unsafe {
            for i in s.chars() {
                NS16550A.add(0).write_volatile(i as u8);
            }
            Ok(())
        }
    }
}

const NS16550A: *mut u8 = 0x1000_0000usize as *mut u8;
fn ns16550a_init() {
    unsafe {
        // 8 bit
        NS16550A.add(3).write_volatile(0b11);
        // FIFO
        NS16550A.add(2).write_volatile(0b1);
        // 关闭中断
        NS16550A.add(1).write_volatile(0b0);
    }
}
