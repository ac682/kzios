#![no_std]

use alloc::vec::Vec;
use rinlib::{
    ipc::message::{peek, send},
    preclude::*,
};

fn main() {
    debug!("Hello, init!");

    if send(4, 114, &[5u8, 1u8, 4u8]) {
        if let Some(digest) = peek() {
            debug!("{}", digest.kind);
            let payload = Vec::<u8>::with_capacity(digest.payload_length);
            
        } else {
            debug!("peek fail");
        }
    } else {
        debug!("send fail");
    }
}
