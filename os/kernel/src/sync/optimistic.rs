use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use spin::once::Once;

use super::Lock;

/// Assuming data is always available for all the threads and has no racing condition
///
/// Suitable for multi-core and single thread (in single-threaded kernel)
pub struct OptimisticLock<T: Sized> {
    data: Once<T>,
}

pub struct OptimisticLockGuard<T: Sized> {
    data: *mut T,
}

impl<T> OptimisticLock<T> {
    pub fn new(data: T) -> Self {
        let cell: Once<T> = Once::new();
        cell.call_once(|| data);
        Self { data: cell }
    }

    pub const fn empty() -> Self {
        Self { data: Once::new() }
    }

    pub fn put(&self, data: T) {
        self.data.call_once(|| data);
    }
}

unsafe impl<T> Sync for OptimisticLock<T> {}

impl<T> Lock<T, OptimisticLockGuard<T>> for OptimisticLock<T> {
    fn lock(&self) -> OptimisticLockGuard<T> {
        OptimisticLockGuard {
            data: self.data.as_mut_ptr(),
        }
    }
}

impl<T> Deref for OptimisticLockGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for OptimisticLockGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}
