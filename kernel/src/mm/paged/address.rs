#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    //
}

impl From<u64> for PhysicalAddress {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<PhysicalAddress> for u64 {
    fn from(val: PhysicalAddress) -> Self {
        val.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub fn vpn(&self) -> (u64, u64, u64) {
        (
            self.0 & 0x1FF000,
            self.0 & 0x3FE00000,
            self.0 & 0x7FC0000000,
        )
    }

    pub fn offset(&self) -> u64 {
        self.0 & 0xFFF
    }
}

impl From<u64> for VirtualAddress {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<VirtualAddress> for u64 {
    fn from(val: VirtualAddress) -> Self {
        val.0
    }
}
