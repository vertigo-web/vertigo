#![allow(clippy::mut_from_ref)]

use std::cell::UnsafeCell;
use std::marker::PhantomData;

struct InnerValueErased {
    data: *mut (),
    drop_fn: unsafe fn(*mut ()),
    eq_fn: Option<unsafe fn(*const (), *const ()) -> bool>,
    fmt_fn: Option<unsafe fn(*const (), &mut std::fmt::Formatter<'_>) -> std::fmt::Result>,
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
            eq_fn: None,
            fmt_fn: None,
        }
    }

    fn new_with_eq<T: PartialEq>(value: T) -> Self {
        let boxed = Box::new(UnsafeCell::new(value));
        let data = Box::into_raw(boxed) as *mut ();

        unsafe fn drop_ptr<T>(ptr: *mut ()) {
            let _ = unsafe { Box::from_raw(ptr as *mut UnsafeCell<T>) };
        }

        unsafe fn eq_ptr<T: PartialEq>(ptr: *const (), other: *const ()) -> bool {
            let this = unsafe { &*(ptr as *const UnsafeCell<T>) };
            let other = unsafe { &*(other as *const T) };
            unsafe { *this.get() == *other }
        }

        Self {
            data,
            drop_fn: drop_ptr::<T>,
            eq_fn: Some(eq_ptr::<T>),
            fmt_fn: None,
        }
    }

    fn new_with_eq_debug<T: PartialEq + std::fmt::Debug>(value: T) -> Self {
        let boxed = Box::new(UnsafeCell::new(value));
        let data = Box::into_raw(boxed) as *mut ();

        unsafe fn drop_ptr<T>(ptr: *mut ()) {
            let _ = unsafe { Box::from_raw(ptr as *mut UnsafeCell<T>) };
        }

        unsafe fn eq_ptr<T: PartialEq>(ptr: *const (), other: *const ()) -> bool {
            let this = unsafe { &*(ptr as *const UnsafeCell<T>) };
            let other = unsafe { &*(other as *const T) };
            unsafe { *this.get() == *other }
        }

        unsafe fn fmt_ptr<T: std::fmt::Debug>(
            ptr: *const (),
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            let this = unsafe { &*(ptr as *const UnsafeCell<T>) };
            unsafe { (*this.get()).fmt(f) }
        }

        Self {
            data,
            drop_fn: drop_ptr::<T>,
            eq_fn: Some(eq_ptr::<T>),
            fmt_fn: Some(fmt_ptr::<T>),
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

    pub fn new_with_eq(value: T) -> InnerValue<T>
    where
        T: PartialEq,
    {
        InnerValue {
            inner: InnerValueErased::new_with_eq(value),
            _phantom: PhantomData,
        }
    }

    pub fn new_with_eq_debug(value: T) -> InnerValue<T>
    where
        T: PartialEq + std::fmt::Debug,
    {
        InnerValue {
            inner: InnerValueErased::new_with_eq_debug(value),
            _phantom: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        unsafe { self.inner.get::<T>() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { self.inner.get_mut::<T>() }
    }

    pub fn is_eq(&self, other: &T) -> bool {
        if let Some(eq_fn) = self.inner.eq_fn {
            unsafe { eq_fn(self.inner.data, other as *const T as *const ()) }
        } else {
            // Fallback for cases where we didn't store eq_fn, but it shouldn't happen if we use ValueMut
            false
        }
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
        if let Some(fmt_fn) = self.inner.fmt_fn {
            unsafe { fmt_fn(self.inner.data, f) }
        } else {
            f.debug_struct("InnerValue")
                .field("value", self.get())
                .finish()
        }
    }
}
