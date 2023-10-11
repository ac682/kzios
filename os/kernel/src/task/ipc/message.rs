use alloc::vec::Vec;
use erhino_shared::{message::MessageDigest, proc::Pid, time::Timestamp};

pub struct Message {
    sender: Pid,
    kind: usize,
    time: Timestamp,
    content: Vec<u8>,
}

impl Message {
    pub fn new(sender: Pid, kind: usize, content: Vec<u8>) -> Self {
        Self {
            sender,
            kind,
            time: 0,
            content,
        }
    }

    pub fn digest(&self) -> MessageDigest {
        MessageDigest::new(self.sender, self.kind, self.time, self.content.len())
    }
}

pub struct Mailbox {
    inbox: Option<Message>,
}

impl Mailbox {
    pub fn new() -> Self {
        Self { inbox: None }
    }

    pub fn available(&self) -> bool {
        self.inbox.is_none()
    }

    pub fn put(&mut self, msg: Message) -> bool {
        if self.inbox.is_some() {
            false
        } else {
            self.inbox = Some(msg);
            true
        }
    }

    pub fn take(&mut self) -> Option<Message> {
        self.inbox.take()
    }
}
