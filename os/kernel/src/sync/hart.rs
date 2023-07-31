use core::{
    hint::spin_loop,
    sync::atomic::{AtomicUsize, Ordering},
};

use erhino_shared::sync::{InteriorLock, InteriorLockMut};

use crate::hart;

pub struct HartLock {
    lock: AtomicUsize,
}

impl HartLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicUsize::new(usize::MAX),
        }
    }
}

impl InteriorLock for HartLock {
    fn is_locked(&self) -> bool {
        let hartid = hart::hartid();
        let locked = self.lock.load(Ordering::Relaxed);
        locked != usize::MAX && locked != hartid
    }

    fn lock(&self) {
        let hartid = hart::hartid();
        while self
            .lock
            .compare_exchange(usize::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|c| c != hartid)
        {
            while self.is_locked() {
                spin_loop()
            }
        }
    }

    fn unlock(&self) {
        self.lock.store(usize::MAX, Ordering::Relaxed);
    }

    fn try_lock(&self) -> bool {
        let hartid = hart::hartid() ;
        match self
            .lock
            .compare_exchange(usize::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => true,
            Err(current) => current == hartid,
        }
    }
}