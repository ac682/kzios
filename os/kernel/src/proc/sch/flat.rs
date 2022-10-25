use alloc::vec::Vec;

use crate::{proc::{Process}, sync::{hart::{HartReadWriteLock}, Lock, ReadWriteLock}};

use super::Scheduler;

pub struct FlatScheduler{
}

impl Scheduler for FlatScheduler{
    fn new() -> Self{
        FlatScheduler {  }
    }
    fn tick(&self){}
}