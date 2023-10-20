use alloc::vec::Vec;

use crate::hart::HartId;

use self::{
    cpu::{Cpu, MmuType},
    generic::GenericDeviceBuilder,
};

pub mod cpu;
pub mod generic;
pub mod memory;

pub struct DeviceMap {
    cpus: Vec<Cpu>,
    // interrupt-controller(plic only)
}

impl DeviceMap {
    pub fn builder() -> DeviceMapBuilder {
        DeviceMapBuilder::empty()
    }

    pub fn cpus(&self) -> &[Cpu] {
        &self.cpus
    }
}

pub struct DeviceMapBuilder {
    cpus: Vec<Cpu>,
    generic: Vec<GenericDeviceBuilder>,
}

impl DeviceMapBuilder {
    pub fn empty() -> Self {
        DeviceMapBuilder {
            cpus: Vec::new(),
            generic: Vec::new(),
        }
    }
    pub fn build(self) -> DeviceMap {
        // TODO: 对 generic builder 中按依赖数量排序从小到大添加到列表，直到最后一个被添加的依赖数量为 0 并被全部添加
        DeviceMap { cpus: self.cpus }
    }

    pub fn cpu(&mut self, hartid: HartId, freq: usize, mmu_type: MmuType) {
        self.cpus.push(Cpu::new(hartid, freq, mmu_type))
    }
}
