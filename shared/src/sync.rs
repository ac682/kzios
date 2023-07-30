use core::ops::{Deref, DerefMut};

use alloc::sync;

pub trait InteriorLock {
    fn is_locked(&self) -> bool;
    fn lock(&self);
    fn try_lock(&self) -> bool;
    fn unlock(&self);
}

pub trait InteriorLockMut: InteriorLock {
    fn lock_mut(&self);
    fn try_lock_mut(&self) -> bool;
}

pub struct DataLock<Data: Sized + Send + Sync, Lock: InteriorLock> {
    inner: Lock,
    data: Data,
}

pub struct DataLockGuard<'lock, Data: Sized + Send + Sync, Lock: InteriorLock> {
    locked: &'lock mut Lock,
    data: *mut Data,
}

unsafe impl<Data: Sized + Send + Sync, Lock: InteriorLock> Sync for DataLock<Data, Lock> {}
unsafe impl<Data: Sized + Send + Sync, Lock: InteriorLock> Send for DataLock<Data, Lock> {}

impl<Data: Sized + Send + Sync, Lock: InteriorLock> DataLock<Data, Lock> {
    pub const fn new(data: Data, lock: Lock) -> Self {
        Self { inner: lock, data }
    }

    pub fn lock(&mut self) -> DataLockGuard<Data, Lock> {
        self.inner.lock();
        DataLockGuard {
            locked: &mut self.inner,
            data: &mut self.data as *mut Data,
        }
    }
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLock> Deref
    for DataLockGuard<'lock, Data, Lock>
{
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLock> DerefMut
    for DataLockGuard<'lock, Data, Lock>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLock> Drop
    for DataLockGuard<'lock, Data, Lock>
{
    fn drop(&mut self) {
        self.locked.unlock();
    }
}

// pub struct ReadWriteDataLock<Data: Sized + Send + Sync, Lock: InteriorLockMut> {
//     inner: Lock,
//     data: Data,
// }

// impl<Data: Sized + Send + Sync, Lock: InteriorLockMut> ReadWriteDataLock<Data, Lock> {
//     pub const fn new(data: Data, lock: Lock) -> Self {
//         Self {
//             inner: lock,
//             data: data,
//         }
//     }
//     pub fn lock(&mut self) -> ReadDataLockGuard<Data, Lock> {
        
//     }
// }

// pub struct ReadDataLockGuard<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> {
//     lock: &'lock Lock,
//     data: *const Data,
// }

// pub struct WriteDataLockGuard<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> {
//     lock: &'lock Lock,
//     data: *mut Data,
// }
