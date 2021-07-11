// FIXME: To be removed in favor of standard RefCell

use std::cell::{RefCell};

pub struct BoxRefCell<T> {
    label: &'static str,
    value: RefCell<T>,
}

impl<T> BoxRefCell<T> {
    pub fn new(value: T, label: &'static str) -> BoxRefCell<T> {
        BoxRefCell {
            value: RefCell::new(value),
            label,
        }
    }

    pub fn get<R>(&self, getter: fn(&T) -> R) -> R {
        let value = self.value.try_borrow();
        match value {
            Ok(value) => {
                let state = &*value;
                getter(state)
            },
            Err(msg) => {
                panic!("Error borrow for '{}', {}", self.label, msg);
            }
        }
    }

    pub fn get_with_context<D, R>(&self, data: D, getter: fn(&T, D) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;
        getter(state, data)
    }

    pub fn change<D, R>(&self, data: D, change_fn: fn(&mut T, D) -> R) -> R {
        let value = self.value.try_borrow_mut();
        match value {
            Ok(value) => {
                let mut state = value;
                change_fn(&mut state, data)
            },
            Err(msg) => {
                panic!("Error mut borrow for '{}', {}", self.label, msg);
            }
        }
    }
}
