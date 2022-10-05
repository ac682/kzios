#![no_std]

extern crate alloc;
extern crate erhino_kernel;

use core::fmt::{Arguments, Result, Write};

use alloc::borrow::ToOwned;
use erhino_kernel::{board::BoardInfo, init, println, env};

fn main() {
    // prepare BoardInfo
    let info = BoardInfo {
        name: "qemu".to_owned(),
        mtimecmp_addr: 0x0200_4000,
    };
    init(info);
    let dtb_addr = env::args()[1] as usize;
    let tree = dtb_parser::device_tree::DeviceTree::from_address(dtb_addr).unwrap();
    todo!("find clint base and store into BoardInfo");
    
}

#[export_name = "write_out"]
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
