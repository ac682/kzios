use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
};

use riscv::{asm::nop, register::mhartid};
use spin::Once;

use super::{InteriorLock, InteriorReadWriteLock};

pub struct HartLock {
    lock: AtomicU64,
}

pub struct HartLockGuard<'lock> {
    locked: &'lock mut AtomicU64,
}

impl HartLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicU64::new(u64::MAX),
        }
    }
}

impl<'lock> InteriorLock<'lock> for HartLock {
    type Guard = HartLockGuard<'lock>;

    fn is_locked(&self) -> bool {
        let hartid = mhartid::read() as u64;
        let locked = self.lock.load(Ordering::Relaxed);
        locked != u64::MAX && locked != hartid
    }

    fn lock(&'lock mut self) -> Self::Guard {
        let hartid = mhartid::read() as u64;
        while self
            .lock
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|c| c != hartid)
        {
            unsafe { nop() };
        }
        Self::Guard {
            locked: &mut self.lock,
        }
    }
}

impl<'lock> Drop for HartLockGuard<'lock> {
    fn drop(&mut self) {
        self.locked.store(u64::MAX, Ordering::Release);
    }
}

pub struct HartReadWriteLock {
    lock: AtomicU64,
}

pub struct HartReadLockGuard<'lock> {
    locked: &'lock AtomicU64,
}

pub struct HartWriteLockGuard<'lock> {
    locked: &'lock AtomicU64,
}

impl HartReadWriteLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicU64::new(u64::MAX),
        }
    }
}

impl<'lock> InteriorLock<'lock> for HartReadWriteLock {
    type Guard = HartReadLockGuard<'lock>;

    fn is_locked(&self) -> bool {
        let hartid = mhartid::read() as u64;
        let locked = self.lock.load(Ordering::Relaxed);
        locked != u64::MAX && locked != hartid
    }

    fn lock(&'lock mut self) -> Self::Guard {
        let hartid = mhartid::read() as u64;
        loop {
            let locked = self.lock.load(Ordering::Relaxed);
            if locked == u64::MAX || locked == hartid {
                break;
            }
        }
        Self::Guard {
            locked: &mut self.lock,
        }
    }
}

impl<'lock> InteriorReadWriteLock<'lock> for HartReadWriteLock {
    type GuardMut = HartWriteLockGuard<'lock>;

    fn lock_mut(&'lock mut self) -> Self::GuardMut {
        let hartid = mhartid::read() as u64;
        while self
            .lock
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|c| c != hartid)
        {
            unsafe { nop() };
        }
        Self::GuardMut {
            locked: &mut self.lock,
        }
    }
}

impl<'lock> Drop for HartReadLockGuard<'lock> {
    fn drop(&mut self) {
        // do nothing for its not locked
    }
}

impl<'lock> Drop for HartWriteLockGuard<'lock> {
    fn drop(&mut self) {
        self.locked.store(u64::MAX, Ordering::Release);
    }
}
