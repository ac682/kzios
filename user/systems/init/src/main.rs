#![no_std]

use rinlib::{prelude::*, proc::fork, shared::proc::ProcessPermission};

extern crate rinlib;

fn main() {
    dbg!("Init not so Ready\n");
    // if let Ok(child) = fork(ProcessPermission::Invalid){
    //     if child != 0{
    //         dbg!("My fork: {}", child);
    //     }else{
    //         dbg!("I died: {}", child);
    //     }
    // }
    loop {
        dbg!("Who am I?");
        for _ in 0..500000 {}
    }
}
