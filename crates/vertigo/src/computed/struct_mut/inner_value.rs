#![allow(clippy::mut_from_ref)]

use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct InnerValue<T> {
    value: UnsafeCell<T>,
}

impl<T> InnerValue<T> {
    pub fn new(value: T) -> InnerValue<T> {
        InnerValue {
            value: UnsafeCell::new(value),
        }
    }
    pub fn get(&self) -> &T {
        unsafe { &*self.value.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}
