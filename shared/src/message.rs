use crate::{proc::Pid, time::Timestamp};

/// Message sender and kind
#[repr(C)]
pub struct MessageDigest {
    pub sender: Pid,
    pub kind: usize,
    pub time: Timestamp,
    pub payload_length: usize,
}

impl MessageDigest {
    pub fn new(sender: Pid, kind: usize, time: Timestamp, length: usize) -> Self {
        Self {
            sender,
            kind,
            time,
            payload_length: length,
        }
    }
}
