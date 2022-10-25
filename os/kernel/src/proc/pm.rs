use alloc::vec::Vec;

use crate::sync::{hart::HartReadWriteLock, Lock, ReadWriteLock};

use super::Process;

static mut PROC_TABLE: HartReadWriteLock<Vec<Process>> = HartReadWriteLock::empty();

pub fn init(){
    unsafe{
        PROC_TABLE.put(Vec::new());
    }
}

pub fn add_process(proc: Process){
    unsafe{
        PROC_TABLE.lock_mut().push(proc);
    }
}