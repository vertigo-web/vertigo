use std::cell::RefCell;

pub struct VecMut<V> {
    data: RefCell<Vec<V>>,
}

impl<V> Default for VecMut<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> VecMut<V> {
    pub fn new() -> VecMut<V> {
        VecMut {
            data: RefCell::new(Vec::new())
        }
    }

    pub fn push(&self, value: V) {
        let mut state = self.data.borrow_mut();
        state.push(value);
    }

    pub fn take(&self) -> Vec<V> {
        let mut state = self.data.borrow_mut();
        std::mem::take(&mut state)
    }

    pub fn for_each(&self, callback: impl Fn(&V)) {
        let state = self.data.borrow();
        for item in state.iter() {
            callback(item);
        }
    }
}
