use std::{cell::RefCell};

#[derive(Debug)]
pub struct ValueMut<T> {
    value: RefCell<T>,
}

impl<T> ValueMut<T> {
    pub fn new(value: T) -> ValueMut<T> {
        ValueMut {
            value: RefCell::new(value)
        }
    }

    pub fn set(&self, value: T) {
        let mut state = self.value.borrow_mut();
        *state = value;
    }

    pub fn map<K>(&self, fun: impl Fn(&T) -> K) -> K {
        let state = self.value.borrow();
        fun(&state)
    }

    pub fn change<R>(&self, change: impl FnOnce(&mut T) -> R) -> R {
        let mut state = self.value.borrow_mut();
        change(&mut state)
    }
}

impl<T: Default> ValueMut<T> {
    pub fn move_to<R>(&self, change: impl Fn(T) -> (T, R)) -> R {
        let mut state = self.value.borrow_mut();
        let prev_state = std::mem::take::<T>(&mut state);
        let (new_state, rest) = change(prev_state);
        let _ = std::mem::replace::<T>(&mut state, new_state);
        rest
    }

    pub fn move_to_void(&self, change: impl Fn(T) -> T) {
        let mut state = self.value.borrow_mut();
        let prev_state = std::mem::take::<T>(&mut state);
        let new_state = change(prev_state);
        let _ = std::mem::replace::<T>(&mut state, new_state);
    }
}

impl<T: Clone> ValueMut<T> {
    pub fn get(&self) -> T {
        let state = self.value.borrow();
        (*state).clone()
    }
}

impl<T: PartialEq> ValueMut<T> {
    pub fn set_and_check(&self, value: T) -> bool {
        let mut state = self.value.borrow_mut();
        let is_change = *state != value;
        *state = value;
        is_change
    }
}