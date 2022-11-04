use core::{
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
};

use riscv::{asm::nop, register::mhartid};
use spin::Once;

use crate::println;

use super::{InteriorLock, InteriorReadWriteLock};

pub struct HartLock {
    lock: AtomicU64,
}

impl HartLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicU64::new(u64::MAX),
        }
    }
}

impl InteriorLock for HartLock {
    fn is_locked(&self) -> bool {
        let hartid = mhartid::read() as u64;
        let locked = self.lock.load(Ordering::Relaxed);
        locked != u64::MAX && locked != hartid
    }

    fn lock(&mut self) {
        let hartid = mhartid::read() as u64;
        while self
            .lock
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|c| c != hartid)
        {
            while self.is_locked() {
                spin_loop()
            }
        }
    }

    fn unlock(&mut self) {
        self.lock.store(u64::MAX, Ordering::Relaxed);
    }
}

pub struct HartReadWriteLock {
    lock: AtomicU64,
}

impl HartReadWriteLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicU64::new(u64::MAX),
        }
    }
}

impl InteriorLock for HartReadWriteLock {
    fn is_locked(&self) -> bool {
        let hartid = mhartid::read() as u64;
        let locked = self.lock.load(Ordering::Relaxed);
        locked != u64::MAX && locked != hartid
    }

    fn lock(&mut self) {
        let hartid = mhartid::read() as u64;
        loop {
            let locked = self.lock.load(Ordering::Relaxed);
            if locked == u64::MAX || locked == hartid {
                break;
            }else{
                spin_loop()
            }
        }
    }

    fn unlock(&mut self) {
        self.lock.store(u64::MAX, Ordering::Relaxed);
    }
}

impl InteriorReadWriteLock for HartReadWriteLock {
    fn lock_mut(&mut self) {
        let hartid = mhartid::read() as u64;
        while self
            .lock
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|c| c != hartid)
        {
            while self.is_locked() {
                spin_loop()
            }
        }
    }
}
