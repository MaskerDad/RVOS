//! Avoid unsafe block to access global variable

use core::cell::{RefCell, RefMut};

/*
    Notice:
    static A: RefCell<i32> = RefCell::new(3);
    fn main() {
        *A.borrow_mut() = 4;
    }
    => error: `RefCell<i32>` cannot be shared between threads safely!!!
*/
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}