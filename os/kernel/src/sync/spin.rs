use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};

use erhino_shared::sync::{InteriorLock, InteriorLockMut};

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
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
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

pub struct ReadWriteSpinLock {
    lock: AtomicIsize,
    wait: AtomicBool,
}

impl ReadWriteSpinLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicIsize::new(0),
            wait: AtomicBool::new(false),
        }
    }

    fn is_waiting(&self) -> bool {
        self.wait.load(Ordering::Relaxed)
    }

    fn set_waiting(&self) {
        self.wait.store(true, Ordering::Relaxed);
    }

    fn clear_waiting(&self) {
        self.wait.store(false, Ordering::Relaxed);
    }
}

impl InteriorLock for ReadWriteSpinLock {
    fn is_locked(&self) -> bool {
        self.lock.fetch_max(0, Ordering::Relaxed) == 0
    }

    fn lock(&self) {
        // 一旦有一把锁尝试 isize::MAX 次就会导致逻辑失效。但是放心 writer 会在 unlock_mut 时恢复状态到初始 isize::MIN 重置重试次数
        while self.is_waiting() {
            spin_loop();
        }
        while self.lock.fetch_add(1, Ordering::Relaxed) < 0 {
            while self.is_locked() {
                spin_loop()
            }
        }
    }

    fn try_lock(&self) -> bool {
        !self.is_waiting() && self.lock.fetch_add(1, Ordering::Relaxed) >= 0
    }

    fn unlock(&self) {
        // 当其他线程持有 writer 锁时调用会炸掉，但是放心，unlock 和 lock_mut 互斥，除非持有者自己调用 lock_mut 然后 unlock
        self.lock.fetch_sub(1, Ordering::Relaxed);
    }
}

impl InteriorLockMut for ReadWriteSpinLock {
    fn lock_mut(&self) {
        while self.is_waiting() {
            spin_loop()
        }
        self.set_waiting();
        while self
            .lock
            .compare_exchange_weak(0, isize::MIN, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                spin_loop()
            }
        }
        self.clear_waiting();
    }

    fn try_lock_mut(&self) -> bool {
        !self.is_waiting()
            && self
                .lock
                .compare_exchange_weak(0, isize::MIN, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
    }

    fn unlock_mut(&self) {
        self.lock.store(0, Ordering::Relaxed);
    }
}
