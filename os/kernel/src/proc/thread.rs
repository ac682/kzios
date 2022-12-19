use alloc::string::String;
use erhino_shared::{proc::{Tid, Signal, ProcessState}, mem::Address};

use crate::trap::TrapFrame;

#[derive(Clone, Copy)]
pub struct SignalControlBlock {
    pub mask: Signal,
    pub pending: Signal,
    pub handler: Address,
    pub backup: TrapFrame,
}

impl Default for SignalControlBlock {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            pending: Default::default(),
            handler: Default::default(),
            backup: TrapFrame::new(),
        }
    }
}

pub struct Thread {
    pub name: String,
    pub tid: Tid,
    pub frame: TrapFrame,
    pub state: ProcessState,
    signal: SignalControlBlock
}

impl Thread {
    // tid is assigned when attached to a process
    pub fn new(name: String, entry_point: Address, stack_address: Address, satp: u64) -> Self {
        let mut trap = TrapFrame::new();
        trap.pc = entry_point as u64;
        trap.x[2] = stack_address as u64;
        trap.satp = satp;
        Self {
            name,
            tid: 0,
            frame: trap,
            state: ProcessState::Ready,
            signal: SignalControlBlock::default()
        }
    }

    pub fn move_to_next_instruction(&mut self) {
        self.frame.pc += 4;
    }
    
    pub fn has_signals_pending(&self) -> bool {
        self.signal.pending > 0
    }

    pub fn queue_signal(&mut self, signal: Signal) {
        self.signal.pending |= signal as Signal;
    }

    pub fn set_signal_handler(&mut self, handler: Address, mask: Signal) {
        self.signal.mask = mask;
        self.signal.handler = handler;
    }

    pub fn enter_signal(&mut self) {
        self.signal.backup = self.frame.clone();
        let mut signal = 0 as Signal;
        let mut pending = self.signal.pending;
        for i in 0..64 {
            if pending & 2 == 1 {
                signal = 1 << i;
                break;
            } else {
                pending >>= 1;
            }
        }

        self.signal.pending &= !signal;
        self.signal.backup.x[10] = signal;
        self.signal.backup.pc = self.signal.handler as u64;

        (self.frame, self.signal.backup) = (self.signal.backup, self.frame);
    }

    pub fn leave_signal(&mut self) {
        (self.frame, self.signal.backup) = (self.signal.backup, self.frame);
    }
}
