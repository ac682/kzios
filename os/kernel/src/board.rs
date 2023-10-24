use core::cell::OnceCell;

use dtb_parser::{
    prop::PropertyValue,
    traits::{FindPropertyValue, HasNamedChildNode, HasNamedProperty},
    DeviceTree,
};
use erhino_shared::mem::Address;

use self::device::{cpu::MmuType, DeviceMap};

pub mod device;

pub static mut BOARD: OnceCell<BoardInfo> = OnceCell::new();

pub struct BoardInfo {
    tree: DeviceTree,
    map: DeviceMap,
    initfs: Option<(Address, usize)>,
}

impl BoardInfo {
    pub fn from_device_tree(tree: DeviceTree) -> Result<Self, ()> {
        let mut initfs: Option<(Address, usize)> = None;
        let mut map = DeviceMap::builder();
        if let Some(chosen) = tree.find_node("/chosen/initfs") {
            if let Some(PropertyValue::Address(addr, len)) = chosen.value("reg") {
                initfs = Some((*addr as usize, *len as usize));
            }
        }
        let mut timebase_frequency: usize = 0;
        if let Some(cpus) = tree.root().find_child("cpus") {
            if let Some(PropertyValue::Integer(timebase)) = cpus.of_value("timebase-frequency") {
                timebase_frequency = *timebase as usize;
            }
            for cpu in cpus
                .nodes()
                .iter()
                .filter(|maybe| maybe.type_name() == "cpu")
            {
                if let Some(PropertyValue::Address(hartid, _)) = cpu.of_value("reg")
                && let Some(PropertyValue::String(mmu)) = cpu.of_value("mmu-type"){
                    let freq = if let Some(PropertyValue::Integer(frequency)) = cpu.of_value("clock-frequency"){
                        *frequency as usize
                    }else{
                        timebase_frequency
                    };
                    if freq != 0{
                        let mmu_type = match mmu.as_str(){
                            "riscv,sv32" => MmuType::Sv32,
                            "riscv,sv39" => MmuType::Sv39,
                            "riscv,sv48" => MmuType::Sv48,
                            "riscv,sv57" => MmuType::Sv57,
                            _ => MmuType::Bare
                        };
                        map.cpu(*hartid as usize, freq, mmu_type);
                    }
                }
            }
        }
        for node in &tree {
            if let Some(PropertyValue::None) = node.of_value("interrupt-controller") {
            } else {
                continue;
            }
            if !match node.of_value("compatible") {
                Some(PropertyValue::String(compatible)) => compatible == "riscv,plic0",
                Some(PropertyValue::Strings(compatible)) => {
                    compatible.iter().any(|c| c == "riscv,plic0")
                }
                _ => false,
            } {
                continue;
            }
            if let Some(PropertyValue::Address(addr, size)) = node.of_value("reg") {
                map.intrc(*addr as Address, *size as usize);
                break;
            }
        }
        map.build().map(|built| BoardInfo {
            initfs,
            tree,
            map: built,
        })
    }

    pub fn initfs(&self) -> Option<(Address, usize)> {
        self.initfs
    }

    pub fn map(&self) -> &DeviceMap {
        &self.map
    }
}

pub fn init(tree: DeviceTree) {
    if let Ok(board) = BoardInfo::from_device_tree(tree) {
        unsafe {
            let _ = BOARD.set(board);
        }
    } else {
        panic!("parsing board info from device tree failed");
    }
}

pub fn this_board() -> &'static BoardInfo {
    unsafe { BOARD.get().unwrap() }
}
