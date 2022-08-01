#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    //
}

impl From<usize> for PhysicalAddress {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl From<PhysicalAddress> for usize {
    fn from(val: PhysicalAddress) -> Self {
        val.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    pub fn vpn(&self) -> (usize, usize, usize) {
        (
            self.0 & 0x1FF000,
            self.0 & 0x3FE00000,
            self.0 & 0x7FC0000000,
        )
    }

    pub fn offset(&self) -> usize {
        self.0 & 0xFFF
    }
}

impl From<usize> for VirtualAddress {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl From<VirtualAddress> for usize {
    fn from(val: VirtualAddress) -> Self {
        val.0
    }
}
