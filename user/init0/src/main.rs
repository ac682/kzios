#![no_std]

#[macro_use]
extern crate kinlib;

use kinlib::println;
use kinlib::syscall::fork;

#[derive(Debug)]
enum Bar{
    Ark
}

#[derive(Debug)]
struct Foo{
    bar: Bar
}

const FOO: Foo = Foo{bar: Bar::Ark};

fn main()
{
    // TODO: 堆的地方没有被mmap,内核不会去自动分配内存,只能是用户程序手动调用mmap获得内存
    println!("Hello, App");
    //let foo =Foo{ bar: Bar::Ark};
    println!("{:?}", FOO);
    // let pid = fork().unwrap();
    // if pid != 0{
    //     println!("{}", pid);
    // }
    loop{}
}
