use erhino_shared::proc::Pid;

pub struct Message {
    sender: Pid,
    kind: usize,
    content: [u8; 64],
    filed: usize,
}

impl Message {
    pub fn new(sender: Pid, kind: usize, content: [u8; 64], filed: usize) -> Self {
        Self {
            sender,
            kind,
            content,
            filed
        }
    }
}

pub struct Mailbox {
    inbox: Option<Message>,
}

impl Mailbox {
    pub fn new() -> Self {
        Self { inbox: None }
    }
}
