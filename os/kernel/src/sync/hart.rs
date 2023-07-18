use core::{
    hint::spin_loop,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::hart;

use super::{InteriorLock, InteriorReadWriteLock};

fn hartid() -> u64 {
    hart::context().hartid
}

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
        let hartid = hartid();
        let locked = self.lock.load(Ordering::Relaxed);
        locked != u64::MAX && locked != hartid
    }

    fn lock(&self) {
        let hartid = hartid();
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

    fn unlock(&self) {
        self.lock.store(u64::MAX, Ordering::Relaxed);
    }

    fn try_lock(&self) -> bool {
        let hartid = hartid();
        match self
            .lock
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => true,
            Err(current) => current == hartid,
        }
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
        let hartid = hartid();
        let locked = self.lock.load(Ordering::Relaxed);
        locked != u64::MAX && locked != hartid
    }

    fn lock(&self) {
        let hartid = hartid();
        loop {
            let locked = self.lock.load(Ordering::Relaxed);
            if locked == u64::MAX || locked == hartid {
                break;
            } else {
                spin_loop()
            }
        }
    }

    fn try_lock(&self) -> bool {
        let hartid = hartid();
        let locked = self.lock.load(Ordering::Relaxed);
        locked == u64::MAX || locked == hartid
    }

    fn unlock(&self) {
        self.lock.store(u64::MAX, Ordering::Relaxed);
    }
}

impl InteriorReadWriteLock for HartReadWriteLock {
    fn lock_mut(&self) {
        let hartid = hartid();
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

    fn try_lock_mut(&self) -> bool {
        let hartid = hartid();
        match self
            .lock
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => true,
            Err(current) => current == hartid,
        }
    }
}

impl Clone for HartReadWriteLock {
    fn clone(&self) -> Self {
        Self {
            lock: AtomicU64::new(self.lock.load(Ordering::Relaxed)),
        }
    }
}
