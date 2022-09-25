#![no_std]

#[macro_use]
extern crate kinlib;

use kinlib::println;
use kinlib::process::set_signal_handler;

fn main()
{
    set_signal_handler(||{});
    println!("Hello, App");
    loop{}
}
