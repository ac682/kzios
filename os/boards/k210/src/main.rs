#![no_std]

use core::fmt::{Arguments, Result, Write};

use alloc::string::ToString;
use erhino_kernel::{prelude::*, proc::Process};

extern crate alloc;

const FIRST: &[u8] = include_bytes!("../../../../artifacts/initfs/driver_sifive_uart");

fn main() {
    let clint_base = 0x02000000;
    let info = BoardInfo {
        name: "kendryte k210".to_string(),
        base_frequency: 400_000_000,
        mswi_address: clint_base,
        mtimer_address: clint_base + 0x4000,
    };
    kernel_init(info);

    println!("K210 with 6MB ram only supports loading one elf(with debug symbols).");
    if let Ok(process) = Process::from_elf(FIRST, "adam") {
        add_flat_process(process);
    } else {
        panic!("process from artifacts has wrong format");
    }
    kernel_main();
}

#[export_name = "board_write"]
pub fn uart_write(args: Arguments) {
    UartHs.write_fmt(args).unwrap();
}

#[export_name = "board_init"]
pub fn board_init() {
    unsafe {
        // 1 stop bits and enable transmit
        SIFIVE_UARTHS.add(2).write(0b11);
        // ie auto reset to zero
        SIFIVE_UARTHS.add(4).write(0);
        // div = CLOCK / BAUD_RATE - 1, 3472 for 115200
        SIFIVE_UARTHS.add(6).write(3472);
    }
}

#[export_name = "board_hart_awake"]
pub fn board_hart_awake() {
    // k210 has no pmp
}

const SIFIVE_UARTHS: *mut u32 = 0x38000000 as *mut u32;

struct UartHs;

impl Write for UartHs {
    fn write_str(&mut self, s: &str) -> Result {
        unsafe {
            for i in s.chars() {
                SIFIVE_UARTHS.add(0).write_volatile(i as u32);
            }
            Ok(())
        }
    }
}
