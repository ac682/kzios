#![no_std]

use rinlib::prelude::*;

extern crate rinlib;

fn main() {
    dbg!("SiFive Uart should not be Ready in qemu\n");
    loop {
        dbg!("But is that real?\n");
        for _ in 0..500000 {}
    }
}
