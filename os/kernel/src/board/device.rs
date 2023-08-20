use alloc::vec::Vec;
use dtb_parser::{
    prop::PropertyValue,
    traits::{FindPropertyValue, HasNamedChildNode},
    DeviceTree,
};

use self::{cpu::Cpu, peripheral::PeripheralKind};

pub mod bus;
pub mod cpu;
pub mod peripheral;

pub struct DeviceMap {
    cpus: Vec<Cpu>,
    peripherals: Vec<PeripheralKind>,
}

impl DeviceMap {
    pub fn from_device_tree(dtb: &DeviceTree) -> Self {
        let mut cpu_table = Vec::<Cpu>::new();
        if let Some(cpus) = dtb.root().find_child("cpus") {
            let timebase_frequency =
                if let Some(PropertyValue::Integer(t)) = cpus.value("timebase-frequency") {
                    Some(*t as usize)
                } else {
                    None
                };
            for cpu in cpus.nodes() {
                if let Some(c) = Cpu::from_device_node(cpu, &timebase_frequency) {
                    cpu_table.push(c);
                }
            }
        }
        let mut peripherals = Vec::<PeripheralKind>::new();
        if let Some(soc) = dtb.root().find_child("soc") {
            for node in soc.nodes() {
                if let Some(peripheral) = PeripheralKind::from_device_node(node) {
                    peripherals.push(peripheral);
                }
            }
        }
        Self {
            cpus: cpu_table,
            peripherals,
        }
    }

    pub fn cpus(&self) -> &[Cpu] {
        &self.cpus
    }

    pub fn peripherals(&self) -> &[PeripheralKind] {
        &self.peripherals
    }
}
