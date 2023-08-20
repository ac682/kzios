use core::fmt::Display;

use alloc::{borrow::ToOwned, string::String};
use dtb_parser::{prop::PropertyValue, traits::FindPropertyValue, DeviceTree};
use spin::Once;

use self::{device::DeviceMap, see::SbiInfo};

pub mod device;
pub mod see;

static mut BOARD: Once<Board> = Once::new();
static mut IS_BOARD_READY: bool = false;

pub struct Board {
    model: String,
    see: SbiInfo,
    tree: DeviceTree,
    devices: DeviceMap,
}

impl Board {
    pub fn from_device_tree(device: DeviceTree) -> Result<Self, ()> {
        let model = if let Some(PropertyValue::String(inner)) = device.root().value("model") {
            inner
        } else {
            "Unknown"
        };
        let map = DeviceMap::from_device_tree(&device);
        Ok(Self {
            model: model.to_owned(),
            see: SbiInfo::new(),
            tree: device,
            devices: map,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn see(&self) -> &SbiInfo {
        &self.see
    }

    pub fn tree(&self) -> &DeviceTree {
        &self.tree
    }

    pub fn devices(&self) -> &DeviceMap {
        &self.devices
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Model: {}", self.model)?;
        writeln!(
            f,
            "Sbi({}.{}): {:?}/{}",
            self.see.spec_version().major,
            self.see.spec_version().minor,
            self.see.impl_id(),
            self.see.impl_version()
        )?;
        writeln!(f, "Device: ")?;
        writeln!(f, "Cpu count: {}", self.devices.cpus().len())?;
        writeln!(f, "Peripheral count: {}", self.devices.peripherals().len())
    }
}

pub fn init(dtb: DeviceTree) {
    let board = Board::from_device_tree(dtb).unwrap();
    unsafe {
        IS_BOARD_READY = true;
        BOARD.call_once(|| board);
    }
}

pub fn this_board() -> &'static Board {
    unsafe { BOARD.get().unwrap() }
}

pub fn is_board_ready() -> bool {
    unsafe { IS_BOARD_READY }
}
