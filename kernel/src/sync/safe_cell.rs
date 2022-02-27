use core::cell::{RefCell, RefMut};

/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uni_processor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.
pub struct SafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for SafeCell<T> {}

impl<T> SafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uni_processor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
