#![no_std]

use rinlib::{
    env,
    ipc::message::{peek, receive, send},
    preclude::*,
};

fn main() {
    debug!("Hello, init!");

    if send(env::pid(), 114, &[5u8, 1u8, 4u8]) {
        if let Some(digest) = peek() {
            debug!("{}: {}", digest.kind, digest.payload_length);
            if let Some(payload) = receive(&digest) {
                debug!("payload: {:?}", payload);
            } else {
                debug!("no payload")
            }
        } else {
            debug!("peek fail");
        }
    } else {
        debug!("send fail");
    }
}
