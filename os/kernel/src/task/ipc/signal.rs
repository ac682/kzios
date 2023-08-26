use core::mem::{size_of, size_of_val};

use erhino_shared::{mem::Address, proc::SignalMap};

use crate::trap::TrapFrame;

pub struct SignalControlBlock {
    x: [u64; 32],
    f: [u64; 32],
    pc: u64,
    mask: SignalMap,
    pending: SignalMap,
    handling: bool,
    complete: bool,
    handler: Option<Address>,
}

impl SignalControlBlock {
    pub fn new() -> Self {
        Self {
            x: [0; 32],
            f: [0; 32],
            pc: 0,
            mask: 0,
            pending: 0,
            handling: false,
            complete: false,
            handler: None,
        }
    }

    pub fn is_accepted(&self, signal: SignalMap) -> bool {
        signal & self.mask != 0
    }

    pub fn has_pending(&self) -> bool {
        self.pending > 0
    }

    pub fn is_handling(&self) -> bool {
        self.handling
    }

    pub fn enqueue(&mut self, signal: SignalMap) {
        self.pending |= signal;
    }

    pub fn dequeue(&mut self) -> SignalMap {
        let mut pending = self.pending;
        let mut signal = 0u64;
        for i in 0..(size_of::<SignalMap>() * 8) {
            if pending & 0b1 == 1 {
                signal = 1 << i;
                break;
            } else {
                pending >>= 1;
            }
        }
        self.pending &= !signal;
        if signal > 0 {
            self.handling = true;
        }
        signal
    }

    pub fn complete(&mut self) {
        self.handling = false;
        self.complete = true;
    }

    pub fn has_complete_uncleared(&self) -> bool {
        self.complete
    }

    pub fn clear_complete(&mut self) {
        self.complete = false;
    }

    pub fn set_handler(&mut self, mask: SignalMap, handler: Address) {
        self.handler = Some(handler);
        self.mask = mask;
    }

    pub fn has_handler(&self) -> bool {
        self.handler.is_some()
    }

    pub fn handler(&self) -> Option<Address> {
        self.handler
    }

    pub fn backup(&mut self, trapframe: &TrapFrame) {
        for i in 0..32 {
            self.x[i] = trapframe.x[i];
            self.f[i] = trapframe.f[i];
        }
        self.pc = trapframe.pc;
    }

    pub fn restore(&self, trapframe: &mut TrapFrame) {
        for i in 0..32 {
            trapframe.x[i] = self.x[i];
            trapframe.f[i] = self.f[i];
        }
        trapframe.pc = self.pc;
    }
}
