use alloc::vec::Vec;
use dtb_parser::{traits::HasNamedChildNode, DeviceTree, prop::PropertyValue};

use self::{bus::BusKind, cpu::Cpu};

pub mod bus;
pub mod cpu;

pub struct DeviceMap {
    cpus: Vec<Cpu>,
    buses: Vec<BusKind>,
}

impl DeviceMap {
    pub fn from_device_tree(dtb: DeviceTree) -> Self{
        let mut cpu_table = Vec::<Cpu>::new();
        if let Some(cpus) = dtb.root().find_child("cpus") {
            let timebase_frequency = if let Some(PropertyValue::Integer(t)) = cpus.value("timebase-frequency"){
                Some(*t as usize)
            }else{
                None
            };
            for cpu in cpus.nodes() {
                {
                    if let Some(c) = Cpu::from_device_node(cpu, &timebase_frequency) {
                        cpu_table.push(c);
                    }
                }
            }
        }
        Self { cpus: cpu_table, buses: Vec::new() }
    }

    pub fn cpus(&self) -> &[Cpu]{
        &self.cpus
    }
}
