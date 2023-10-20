use crate::hart::HartId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MmuType {
    Bare,
    Sv32,
    Sv39,
    Sv48,
    Sv57,
}

pub struct Cpu {
    id: HartId,
    frequency: usize,
    mmu: MmuType,
}

impl Cpu {
    pub fn new(id: HartId, freq: usize, mmu_type: MmuType) -> Cpu {
        Cpu {
            id,
            frequency: freq,
            mmu: mmu_type,
        }
    }

    pub fn id(&self) -> HartId {
        self.id
    }

    pub fn freq(&self) -> usize {
        self.frequency
    }

    pub fn mmu(&self) -> MmuType {
        self.mmu
    }
}
