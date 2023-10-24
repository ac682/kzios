use erhino_shared::mem::Address;

// plic
pub struct InterruptController {
    address: Address,
    size: usize,
}

impl InterruptController {
    pub const fn new(addr: Address, size: usize) -> Self {
        Self {
            address: addr,
            size,
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
