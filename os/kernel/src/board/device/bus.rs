use alloc::{borrow::ToOwned, string::String, vec::Vec};
use dtb_parser::{node::DeviceTreeNode, prop::PropertyValue, traits::FindPropertyValue};

pub enum BusKind {
    Spi(Spi, Vec<SpiDeviceKind>),
}

pub enum SpiDeviceKind {
    Mmc(SpiMmc),
}

impl BusKind {
    pub fn from_device_node(
        node: &DeviceTreeNode,
        name: String,
        compatible: String,
    ) -> Option<Self> {
        match name.as_str() {
            "spi" => Spi::from_device_node(node, compatible).map(|(s, d)| BusKind::Spi(s, d)),
            _ => None,
        }
    }
}

pub struct Spi {
    compatible: String,
    address: usize,
    length: usize,
    intr_parent_id: usize,
}

impl Spi {
    pub fn from_device_node(
        node: &DeviceTreeNode,
        compatible: String,
    ) -> Option<(Self, Vec<SpiDeviceKind>)> {
        let mut children = Vec::<SpiDeviceKind>::new();
        let (address, length) = if let Some(PropertyValue::Address(a, l)) = node.value("reg") {
            (*a as usize, *l as usize)
        } else {
            return None;
        };
        let intr_parent_id =
            if let Some(PropertyValue::PHandle(value)) = node.value("interrupt-parent") {
                *value as usize
            } else {
                return None;
            };
        Self::parse_children(node, &mut children);
        Some((
            Self {
                compatible,
                address,
                length,
                intr_parent_id,
            },
            children,
        ))
    }

    fn parse_children(node: &DeviceTreeNode, container: &mut Vec<SpiDeviceKind>) {
        for i in node.nodes() {
            if let Some(device) = match i.name() {
                "mmc" => SpiMmc::from_device_node(i).map(|s| SpiDeviceKind::Mmc(s)),
                _ => None,
            } {
                container.push(device);
            }
        }
    }
}

pub struct SpiMmc {
    compatible: String,
}

impl SpiMmc {
    pub fn from_device_node(node: &DeviceTreeNode) -> Option<SpiMmc> {
        let compatible = if let Some(PropertyValue::String(value)) = node.value("compatible") {
            (*value).to_owned()
        } else {
            return None;
        };
        Some(Self { compatible })
    }
}
