use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use super::InteriorLock;

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
            .swap(true, Ordering::Acquire)
        {
            while self.is_locked() {
                spin_loop()
            }
        }
    }

    fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
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
