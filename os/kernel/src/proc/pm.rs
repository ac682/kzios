use crate::sync::hart::HartLock;

use super::ProcessTable;

static mut PROC_TABLE: HartLock<ProcessTable> = HartLock::empty();

pub fn init() {
    unsafe {
        PROC_TABLE.put(ProcessTable::new());
    }
}
