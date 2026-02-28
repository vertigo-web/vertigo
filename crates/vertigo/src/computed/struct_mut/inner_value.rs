#![allow(clippy::mut_from_ref)]

use std::cell::UnsafeCell;
use std::marker::PhantomData;

struct InnerValueErased {
    data: *mut (),
    drop_fn: unsafe fn(*mut ()),
}

impl InnerValueErased {
    fn new<T>(value: T) -> Self {
        let boxed = Box::new(UnsafeCell::new(value));
        let data = Box::into_raw(boxed) as *mut ();

        unsafe fn drop_ptr<T>(ptr: *mut ()) {
            let _ = unsafe { Box::from_raw(ptr as *mut UnsafeCell<T>) };
        }

        Self {
            data,
            drop_fn: drop_ptr::<T>,
        }
    }

    unsafe fn get<T>(&self) -> &T {
        let ptr = self.data as *mut UnsafeCell<T>;
        unsafe { &*(*ptr).get() }
    }

    unsafe fn get_mut<T>(&self) -> &mut T {
        let ptr = self.data as *mut UnsafeCell<T>;
        unsafe { &mut *(*ptr).get() }
    }
}

impl Drop for InnerValueErased {
    fn drop(&mut self) {
        unsafe {
            (self.drop_fn)(self.data);
        }
    }
}

pub struct InnerValue<T> {
    inner: InnerValueErased,
    _phantom: PhantomData<T>,
}

impl<T> InnerValue<T> {
    pub fn new(value: T) -> InnerValue<T> {
        InnerValue {
            inner: InnerValueErased::new(value),
            _phantom: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        unsafe { self.inner.get::<T>() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { self.inner.get_mut::<T>() }
    }

    pub fn into_inner(self) -> T {
        let data = self.inner.data as *mut UnsafeCell<T>;
        let inner_value = unsafe {
            let boxed = Box::from_raw(data);
            boxed.into_inner()
        };
        // Forget the erased part so it doesn't double-drop
        std::mem::forget(self.inner);
        inner_value
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for InnerValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerValue")
            .field("value", self.get())
            .finish()
    }
}
