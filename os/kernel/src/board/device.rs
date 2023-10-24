use alloc::vec::Vec;
use erhino_shared::mem::Address;

use crate::hart::HartId;

use self::{
    cpu::{Cpu, MmuType},
    generic::GenericDeviceBuilder,
    intrc::InterruptController,
};

pub mod cpu;
pub mod generic;
pub mod intrc;
pub mod memory;

pub struct DeviceMap {
    cpus: Vec<Cpu>,
    intrc: InterruptController, // interrupt-controller(plic only)
}

impl DeviceMap {
    pub fn builder() -> DeviceMapBuilder {
        DeviceMapBuilder::empty()
    }

    pub fn cpus(&self) -> &[Cpu] {
        &self.cpus
    }

    pub fn intrc(&self) -> &InterruptController{
        &self.intrc
    }
}

pub struct DeviceMapBuilder {
    cpus: Vec<Cpu>,
    generic: Vec<GenericDeviceBuilder>,
    intrc: Option<InterruptController>,
}

impl DeviceMapBuilder {
    pub fn empty() -> Self {
        DeviceMapBuilder {
            cpus: Vec::new(),
            generic: Vec::new(),
            intrc: None,
        }
    }
    pub fn build(self) -> Result<DeviceMap, ()> {
        // TODO: 对 generic builder 中按依赖数量排序从小到大添加到列表，直到最后一个被添加的依赖数量为 0 并被全部添加
        if let Some(intrc) = self.intrc {
            Ok(DeviceMap {
                cpus: self.cpus,
                intrc,
            })
        } else {
            Err(())
        }
    }

    pub fn cpu(&mut self, hartid: HartId, freq: usize, mmu_type: MmuType) -> &mut Self {
        self.cpus.push(Cpu::new(hartid, freq, mmu_type));
        self
    }

    pub fn intrc(&mut self, addr: Address, size: usize) -> &mut Self {
        self.intrc = Some(InterruptController::new(addr, size));
        self
    }
}
