use core::ops::{Deref, DerefMut};

use spin::once::Once;

use super::Lock;

/// Assuming data is always available for all the threads and has no racing condition
///
/// Suitable for multi-core and single thread (in single-threaded kernel)
pub struct OptimisticLock<T: Sized> {
    data: Option<T>,
}

pub struct OptimisticLockGuard<T: ?Sized> {
    data: *mut T,
}

impl<T> OptimisticLock<T> {
    pub const  fn new(data: T) -> Self {
        Self { data: Some(data) }
    }

    pub const fn empty() -> Self {
        Self { data: None }
    }

    pub fn put(&mut self, data: T) {
        self.data = Some(data);
    }
}

unsafe impl<T> Sync for OptimisticLock<T> {}

impl<'a, T> Lock<'a, T, OptimisticLockGuard<T>> for OptimisticLock<T> {
    fn is_locked(&self) -> bool{
        false
    }
    fn lock(&mut self) -> OptimisticLockGuard<T> {
        OptimisticLockGuard {
            data: self.data.as_mut().unwrap(),
        }
    }

    unsafe fn access(&self) -> *const T{
        self.data.as_ref().unwrap()
    }

    unsafe fn access_mut(&mut self) -> *mut T {
        self.data.as_mut().unwrap()
    }
}

impl<T: ?Sized> Deref for OptimisticLockGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> DerefMut for OptimisticLockGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}
