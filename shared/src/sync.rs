use core::ops::{Deref, DerefMut};

/// Lock trait that locks
pub trait InteriorLock {
    /// True if locked, no blocking
    fn is_locked(&self) -> bool;
    /// Lock if unlocked, block if locked
    fn lock(&self);
    /// Lock if unlocked, no blocking if locked
    fn try_lock(&self) -> bool;
    /// Unlock if locked, do nothing if unlocked
    fn unlock(&self);
}

/// Lock trait that can lock as mut
pub trait InteriorLockMut: InteriorLock {
    /// Lock for mut resource if unlocked, block if locked
    fn lock_mut(&self);
    /// Lock for mut resource if unlocked, no blocking if locked
    fn try_lock_mut(&self) -> bool;
    /// Unlock for mut resource if locked, do nothing if unlocked
    fn unlock_mut(&self);
}

/// Holds data and unlock when resource is dropped
/// RAII
pub struct DataLock<Data: Sized + Send + Sync, Lock: InteriorLock> {
    inner: Lock,
    data: Data,
}

unsafe impl<Data: Sized + Send + Sync, Lock: InteriorLock> Sync for DataLock<Data, Lock> {}
unsafe impl<Data: Sized + Send + Sync, Lock: InteriorLock> Send for DataLock<Data, Lock> {}

impl<Data: Sized + Send + Sync, Lock: InteriorLock> DataLock<Data, Lock> {
    /// Create a new DataLock with the specific lock and data type
    pub const fn new(data: Data, lock: Lock) -> Self {
        Self { inner: lock, data }
    }

    /// Wraps data and returns a container unlocks when dropped
    pub fn lock(&mut self) -> DataLockGuard<Data, Lock> {
        self.inner.lock();
        DataLockGuard {
            locked: &mut self.inner,
            data: &mut self.data as *mut Data,
        }
    }
}

/// Guard for DataLock
pub struct DataLockGuard<'lock, Data: Sized + Send + Sync, Lock: InteriorLock> {
    locked: &'lock mut Lock,
    data: *mut Data,
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

/// Holds data and unlock when dropped.
/// RAII
/// [Self::lock] blocks only if there is a writer.
/// [Self::lock_mut] blocks if there are readers or writers
pub struct ReadWriteDataLock<Data: Sized + Send + Sync, Lock: InteriorLockMut> {
    inner: Lock,
    data: Data,
}

unsafe impl<Data: Sized + Send + Sync, Lock: InteriorLockMut> Sync
    for ReadWriteDataLock<Data, Lock>
{
}
unsafe impl<Data: Sized + Send + Sync, Lock: InteriorLockMut> Send
    for ReadWriteDataLock<Data, Lock>
{
}

impl<Data: Sized + Send + Sync, Lock: InteriorLockMut> ReadWriteDataLock<Data, Lock> {
    /// Create a new DataLock with the specific lock and data type
    pub const fn new(data: Data, lock: Lock) -> Self {
        Self { inner: lock, data }
    }

    /// Wraps data and returns a container unlocks when dropped
    pub fn lock(&self) -> ReadDataLockGuard<Data, Lock> {
        self.inner.lock();
        ReadDataLockGuard {
            locked: &self.inner,
            data: &self.data as *const Data,
        }
    }

    /// Wraps data and returns a container unlocks when dropped
    pub fn lock_mut(&mut self) -> WriteDataLockGuard<Data, Lock> {
        self.inner.lock_mut();
        WriteDataLockGuard {
            locked: &mut self.inner,
            data: &mut self.data as *mut Data,
        }
    }

    /// Get data without lock check
    pub unsafe fn access_unsafe(&self) -> &Data{
        &self.data
    }
}

/// Guard for Reader lock
pub struct ReadDataLockGuard<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> {
    locked: &'lock Lock,
    data: *const Data,
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> Deref
    for ReadDataLockGuard<'lock, Data, Lock>
{
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> Drop
    for ReadDataLockGuard<'lock, Data, Lock>
{
    fn drop(&mut self) {
        self.locked.unlock();
    }
}

/// Guard for Writer lock
pub struct WriteDataLockGuard<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> {
    locked: &'lock mut Lock,
    data: *mut Data,
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> Deref
    for WriteDataLockGuard<'lock, Data, Lock>
{
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> DerefMut
    for WriteDataLockGuard<'lock, Data, Lock>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'lock, Data: Sized + Send + Sync, Lock: InteriorLockMut> Drop
    for WriteDataLockGuard<'lock, Data, Lock>
{
    fn drop(&mut self) {
        self.locked.unlock_mut();
    }
}
