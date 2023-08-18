use core::fmt::Display;

use alloc::{borrow::ToOwned, string::String};
use dtb_parser::{prop::PropertyValue, DeviceTree};
use spin::Once;

use crate::info;

use self::{see::SbiInfo, device::DeviceMap};

pub mod see;
pub mod device;

static mut BOARD: Once<Board> = Once::new();

pub struct Board {
    model: String,
    see: SbiInfo,
    devices: DeviceMap
}

impl Board {
    pub fn from_device_tree(device: DeviceTree) -> Result<Self, ()> {
        let model = if let Some(PropertyValue::String(inner)) = device.root().value("model") {
            inner
        } else {
            "Unknown"
        };
        Ok(Self {
            model: model.to_owned(),
            see: SbiInfo::new(),
            devices: DeviceMap::from_device_tree(device)
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn see(&self) -> &SbiInfo {
        &self.see
    }

    pub fn devices(&self) -> &DeviceMap{
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
        writeln!(f, "Cpu Count: {}", self.devices.cpus().len())
    }
}

pub fn init(dtb: DeviceTree){
    let board = Board::from_device_tree(dtb).unwrap();
    unsafe {
        BOARD.call_once(|| board);
        let reference = BOARD.get().unwrap();
        info!("Board information\n{}", reference);
    }
}

pub fn this_board() -> &'static Board {
    unsafe { BOARD.get().unwrap() }
}
