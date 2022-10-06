use core::{cell::UnsafeCell, sync::atomic::AtomicBool};

pub struct HartLock<T: Sized> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
}

pub struct HartLockGuard<'a, T: Sized> {
    data: *mut T,
    lock: &'a mut AtomicBool,
}
