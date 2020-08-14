use std::cell::RefCell;

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
        getter(&state)
    }

    pub fn getWithContext<D, R>(&self, data: D, getter: fn(&T, D) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;
        getter(&state, data)
    }

    pub fn changeNoParams<R>(&self, changeFn: fn(&mut T) -> R) -> R {
        let value = self.value.borrow_mut();
        let mut state = value;
        changeFn(&mut state)
    }

    pub fn change<D, R>(&self, data: D, changeFn: fn(&mut T, D) -> R) -> R {
        let value = self.value.borrow_mut();
        let mut state = value;
        changeFn(&mut state, data)
    }
}