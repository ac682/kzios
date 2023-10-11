use core::{mem::size_of, slice::from_raw_parts};

use erhino_shared::{message::MessageDigest, proc::Pid};

use crate::call::{sys_peek, sys_send};

pub fn send(target: Pid, kind: usize, payload: &[u8]) -> bool {
    unsafe { sys_send(target, kind, payload) }.is_ok()
}

pub fn peek() -> Option<MessageDigest> {
    let digest = MessageDigest::new(0, 0, 0, 0);
    if let Ok(true) = unsafe {
        sys_peek(from_raw_parts(
            (&digest as *const MessageDigest) as *const u8,
            size_of::<MessageDigest>(),
        ))
    } {
        Some(digest)
    } else {
        None
    }
}
