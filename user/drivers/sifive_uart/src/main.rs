#![no_std]

use core::hint::spin_loop;

use rinlib::prelude::*;

extern crate rinlib;

fn main() {
    dbg!("SiFive Uart should not be Ready in qemu\n");
    loop {
        dbg!("But is that real?\n");
        for _ in 0..10000000 {
            spin_loop();
        }
    }
}
