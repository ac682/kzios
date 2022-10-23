use erhino_shared::Pid;

use crate::sync::{
    hart::{HartLock, HartReadLockGuard, HartReadWriteLock, HartWriteLockGuard},
    optimistic::OptimisticLock,
    Lock, ReadWriteLock,
};

use super::ProcessTable;
static mut PROC_TABLE: HartReadWriteLock<ProcessTable> = HartReadWriteLock::empty();

pub fn init() {
    unsafe {
        PROC_TABLE.put(ProcessTable::new());
    }
}

pub fn table<'table>() -> HartReadLockGuard<ProcessTable<'table>> {
    unsafe { PROC_TABLE.lock() }
}

pub fn table_mut<'a, 'table: 'static>() -> HartWriteLockGuard<'a, ProcessTable<'table>>{
    unsafe{ PROC_TABLE.lock_mut()}
}