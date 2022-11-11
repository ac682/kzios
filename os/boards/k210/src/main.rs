#![no_std]

use core::fmt::{Arguments, Write, Result};

use alloc::string::ToString;
use erhino_kernel::{
    prelude::*,
    proc::Process,
};

extern crate alloc;

const FIRST: &[u8] = include_bytes!("../../../../artifacts/initfs/user_init");

fn main() {
    let clint_base = 0x02000000;
    let info = BoardInfo{
        name: "kendryte k210".to_string(),
        base_frequency: 400_000_000,
        mswi_address: clint_base,
        mtimer_address: clint_base + 0x4000
    };
    kernel_init(info);

    println!("K210 with 8MB ram only supports loading one elf(with debug symbols).");
    if let Ok(process) = Process::from_elf(FIRST, "test") {
        add_flat_process(process);
    } else {
        panic!("process from artifacts has wrong format");
    }
    kernel_main();
}

// 设备被我换成了 NS16550a 方便写代码

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
