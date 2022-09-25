#![no_std]

#[macro_use]
extern crate kinlib;

use kinlib::println;
use kinlib::syscall::fork;

fn main()
{
    println!("Hello, App");
    let pid = fork().unwrap();
    if pid != 0{
        println!("{}", pid);
    }
    loop{}
}
