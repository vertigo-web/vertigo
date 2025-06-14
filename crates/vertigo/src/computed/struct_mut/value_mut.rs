use super::inner_value::InnerValue;

#[derive(Debug)]
pub struct ValueMut<T> {
    value: InnerValue<T>,
}

impl<T> ValueMut<T> {
    pub fn new(value: T) -> ValueMut<T> {
        ValueMut {
            value: InnerValue::new(value),
        }
    }

    pub fn set(&self, value: T) {
        *self.value.get_mut() = value;
    }

    pub fn map<K>(&self, fun: impl Fn(&T) -> K) -> K {
        fun(self.value.get())
    }

    pub fn change<R>(&self, change: impl FnOnce(&mut T) -> R) -> R {
        change(self.value.get_mut())
    }
}

impl<T: Default> ValueMut<T> {
    pub fn move_to<R>(&self, change: impl Fn(T) -> (T, R)) -> R {
        let state = self.value.get_mut();
        let prev_state = std::mem::take::<T>(state);
        let (new_state, rest) = change(prev_state);
        let _ = std::mem::replace::<T>(state, new_state);
        rest
    }

    pub fn move_to_void(&self, change: impl Fn(T) -> T) {
        let state = self.value.get_mut();
        let prev_state = std::mem::take::<T>(state);
        let new_state = change(prev_state);
        let _ = std::mem::replace::<T>(state, new_state);
    }
}

impl<T: Clone> ValueMut<T> {
    pub fn get(&self) -> T {
        // let state = self.value.borrow();
        let state = self.value.get();
        (*state).clone()
    }
}

impl<T: PartialEq> ValueMut<T> {
    pub fn set_if_changed(&self, value: T) -> bool {
        let state = self.value.get_mut();
        if *state != value {
            *state = value;
            true
        } else {
            false
        }
    }
}
