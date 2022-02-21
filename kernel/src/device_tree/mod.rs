use crate::device_tree::model::DeviceTree;

mod model;

pub fn init(addr: usize) {
    let tree = DeviceTree::new(addr);
}
