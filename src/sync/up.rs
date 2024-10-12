use core::cell::{Ref, RefCell, RefMut};

/// Warning: We should only use it in uniprocessor.
pub struct UPCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPCell<T> {}

impl<T> UPCell<T> {
    /// Only uniprocessor
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// Exclusive access to the inner data.
    pub fn exclusive_access(&self) -> RefMut<'_, T> { self.inner.borrow_mut() }

    /// Shared access to the inner data.
    pub fn shared_access(&self) -> Ref<'_, T> { self.inner.borrow() }
}