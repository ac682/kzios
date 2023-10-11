use erhino_shared::{proc::ExecutionState, mem::Address};

use super::ipc::message::Mailbox;

pub struct Thread {
    pub entry_point: Address,
    pub state: ExecutionState,
    pub mailbox: Mailbox
}

impl Thread {
    pub fn new(entry: Address) -> Self {
        Self {
            entry_point: entry,
            state: ExecutionState::Ready,
            mailbox: Mailbox::new()
        }
    }
}
