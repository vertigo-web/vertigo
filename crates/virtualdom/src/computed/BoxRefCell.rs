use std::cell::{RefCell};


pub struct BoxRefCell<T> {
    value: RefCell<T>,
}

impl<T> BoxRefCell<T> {
    pub fn new(value: T) -> BoxRefCell<T> {
        BoxRefCell {
            value: RefCell::new(value),
        }
    }

    pub fn get<R>(&self, getter: fn(&T) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;
        let result = getter(&state);
        result
    }

    pub fn getWithContext<D, R>(&self, data: D, getter: fn(&T, D) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;
        let result = getter(&state, data);
        result
    }

    pub fn changeNoParams<R>(&self, changeFn: fn(&mut T) -> R) -> R {
        let value = self.value.borrow_mut();
        let mut state = value;
        let result = changeFn(&mut state);
        result
    }

    pub fn change<D, R>(&self, data: D, changeFn: fn(&mut T, D) -> R) -> R {
        let value = self.value.borrow_mut();
        let mut state = value;
        let result = changeFn(&mut state, data);
        result
    }
}