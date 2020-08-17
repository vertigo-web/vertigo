use std::rc::Rc;

use crate::lib::{
    BoxRefCell::BoxRefCell,
};

pub struct BoxValue<T: Copy> {
    value: Rc<BoxRefCell<T>>,
}

impl<T: Copy> BoxValue<T> {
    pub fn new(value: T) -> BoxValue<T> {
        BoxValue {
            value: Rc::new(BoxRefCell::new(value))
        }
    }

    pub fn get(&self) -> T {
        self.value.get(|state| {
            *state
        })
    }

    pub fn set(&self, value: T) {
        self.value.change(value, |state, value| {
            *state = value;
        })
    }

    pub fn change<F: Fn(&mut T)>(&self, change: F) {
        self.value.change(change, |state, change| {
            change(state);
        })
    }
}

impl<T: Copy> Clone for BoxValue<T> {
    fn clone(&self) -> Self {
        BoxValue {
            value: self.value.clone()
        }
    }
}