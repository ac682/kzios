use alloc::borrow::ToOwned;
use dtb_parser::{node::DeviceTreeNode, prop::PropertyValue, traits::FindPropertyValue};

use super::bus::BusKind;

pub enum PeripheralKind {
    InterruptController(usize),
    Clock(usize),
    Serial,
    Bus(BusKind),
    Interface,
}

impl PeripheralKind {
    pub fn from_device_node(node: &DeviceTreeNode) -> Option<Self> {
        let name = node.type_name().to_owned();
        let compatible = if let Some(PropertyValue::String(value)) = node.value("compatible") {
            (*value).to_owned()
        } else {
            return None;
        };
        match name.as_str() {
            "spi" | "i2c" => {
                BusKind::from_device_node(node, name, compatible).map(|b| PeripheralKind::Bus(b))
            }
            _ => None
        }
    }
}
