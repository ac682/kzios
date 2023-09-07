use core::{
    hint::spin_loop,
    sync::atomic::{Ordering, AtomicBool, AtomicPtr}, ptr::{null_mut, null},
};

use alloc::boxed::Box;
use erhino_shared::sync::InteriorLock;

pub struct Ticket {
    locked: bool,
    next: *mut Ticket,
}

impl Ticket {
    pub const fn new() -> Self {
        Self {
            locked: true,
            next: null_mut(),
        }
    }
}

pub struct SpinLock {
    tail: AtomicPtr<Ticket>,
    owned: AtomicPtr<Ticket>,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            tail: AtomicPtr::new(null_mut()),
            owned: AtomicPtr::new(null_mut()),
        }
    }
}

impl InteriorLock for SpinLock {
    fn is_locked(&self) -> bool {
        self.tail.load(Ordering::Acquire) != self.owned.load(Ordering::Acquire)
    }

    fn lock(&self) {
        // 泄露 Ticket 到堆，unlock 的时候回收
        let node = Box::into_raw(Box::new(Ticket::new()));
        let prev = self.tail.swap(node, Ordering::Acquire);
        if !prev.is_null() {
            unsafe {
                (*prev).next = node;
                while (*node).locked {
                    spin_loop()
                }
            }
        }
        self.owned.store(node, Ordering::Relaxed);
    }

    fn try_lock(&self) -> bool {
        let node = Box::into_raw(Box::new(Ticket::new()));
        let prev = self.tail.load(Ordering::Acquire);
        if prev.is_null() {
            if self
                .tail
                .compare_exchange(prev, node, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                self.owned.store(node, Ordering::Relaxed);
                return true;
            }
        }
        unsafe { drop(Box::from_raw(node)) };
        return false;
    }

    fn unlock(&self) {
        let self_ptr = self.owned.load(Ordering::Relaxed);
        match self
            .tail
            .compare_exchange(self_ptr, null_mut(), Ordering::Release, Ordering::Relaxed)
        {
            Ok(owned) => unsafe {
                drop(Box::from_raw(owned));
            },
            Err(_) => unsafe {
                let owned = &mut (*self_ptr);
                while owned.next.is_null() {
                    spin_loop()
                }
                let succ = &mut (*owned.next);
                drop(Box::from_raw(owned));
                succ.locked = false;
            },
        }
    }
}
