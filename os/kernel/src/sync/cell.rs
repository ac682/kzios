use core::{cell::UnsafeCell, ops::{Deref, DerefMut}};

// 为不可变对象提供内部可变性，只有在同一个 hart 下是安全的， 所以都被用来作为 Hart 对象的内部元素包装以绕过 rust 的安全检查
pub struct UniProcessCell<T: Sized> {
    data: UnsafeCell<T>,
}

impl<T> UniProcessCell<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &(*self.data.get()) }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (*self.data.get()) }
    }
}

impl<T> Deref for UniProcessCell<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for UniProcessCell<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}