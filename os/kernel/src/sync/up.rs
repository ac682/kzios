use core::{
    cell::{RefCell, UnsafeCell},
    ops::{Deref, DerefMut},
};

pub struct UpSafeCell<T: Sized> {
    data: UnsafeCell<T>,
}

impl<T: Sized> UpSafeCell<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.data.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T: Sized> Deref for UpSafeCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}

impl<T: Sized> DerefMut for UpSafeCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.get_mut()
    }
}

unsafe impl<T: Sized> Sync for UpSafeCell<T> {}
