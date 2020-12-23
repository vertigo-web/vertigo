use std::cell::{RefCell};


pub struct BoxRefCell<T> {
    value: RefCell<Option<T>>,
}

impl<T> BoxRefCell<T> {
    pub fn new(value: T) -> BoxRefCell<T> {
        BoxRefCell {
            value: RefCell::new(Some(value)),
        }
    }

    pub fn get<R>(&self, getter: fn(&T) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;

        if let Some(state) = state {
            getter(&state)
        } else {
            unreachable!();
        }
    }

    pub fn get_with_context<D, R>(&self, data: D, getter: fn(&T, D) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;

        if let Some(state) = state {
            getter(&state, data)
        } else {
            unreachable!();
        }
    }

    pub fn change_no_params<R>(&self, change_fn: fn(&mut T) -> R) -> R {
        let mut value = self.value.borrow_mut();
        let state = &mut *value;

        if let Some(state) = state {
            change_fn(state)
        } else {
            unreachable!();
        }
    }

    pub fn change<D, R>(&self, data: D, change_fn: fn(&mut T, D) -> R) -> R {
        let mut value = self.value.borrow_mut();
        let state = &mut *value;

        if let Some(state) = state {
            change_fn(state, data)
        } else {
            unreachable!();
        }
    }
    
    pub fn replace<D, R>(&self, data: D, change: fn(T, D) -> T) {
        let mut value = self.value.borrow_mut();
        let state = &mut *value;

        let state_inner = std::mem::replace(state, None);

        let new_inner = if let Some(state_change) = state_inner {
            change(state_change, data)
        } else {
            unreachable!();
        };

        let _ = std::mem::replace(state, Some(new_inner));
    }
}
