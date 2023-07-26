use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use erhino_shared::sync::{InteriorLock, InteriorReadWriteLock};

/// spin::Mutex dose not work well, so I made my own one
pub struct SpinLock {
    lock: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
        }
    }
}

impl InteriorLock for SpinLock {
    fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    fn lock(&self) {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                spin_loop()
            }
        }
    }

    fn unlock(&self) {
        self.lock.store(false, Ordering::Relaxed);
    }

    fn try_lock(&self) -> bool {
        match self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

pub struct SpinReadWriteLock {
    lock: AtomicBool,
}

impl SpinReadWriteLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
        }
    }
}

impl InteriorLock for SpinReadWriteLock {
    fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    fn lock(&self) {
        loop {
            let locked = self.lock.load(Ordering::Relaxed);
            if !locked {
                break;
            } else {
                spin_loop()
            }
        }
    }

    fn try_lock(&self) -> bool {
        let locked = self.lock.load(Ordering::Relaxed);
        !locked
    }

    fn unlock(&self) {
        self.lock.store(false, Ordering::Relaxed);
    }
}

impl InteriorReadWriteLock for SpinReadWriteLock {
    fn lock_mut(&self) {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                spin_loop()
            }
        }
    }

    fn try_lock_mut(&self) -> bool {
        self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }
}