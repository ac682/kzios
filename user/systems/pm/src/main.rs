#![no_std]

use core::hint::spin_loop;

use rinlib::preclude::*;

fn main() {
    debug!("Hello, pm!");
    loop {
        spin_loop()
    }
}