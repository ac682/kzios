#![no_std]

#[macro_use]
extern crate kinlib;

use kinlib::println;
use kinlib::process::set_signal_handler;
use kinlib::syscall::fork;

fn main()
{
    set_signal_handler(||{});
    println!("Hello, App");
    let pid = fork().unwrap();
    if pid != 0{
        println!("{}", pid);
    }
    loop{}
}
