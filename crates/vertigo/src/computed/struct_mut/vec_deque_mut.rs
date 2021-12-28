use std::cell::RefCell;
use std::collections::VecDeque;

pub struct VecDequeMut<V> {
    data: RefCell<VecDeque<V>>,
}


impl<V> Default for VecDequeMut<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> VecDequeMut<V> {
    pub fn new() -> VecDequeMut<V> {
        VecDequeMut {
            data: RefCell::new(VecDeque::new())
        }
    }

    pub fn push(&self, value: V) {
        let mut state = self.data.borrow_mut();
        state.push_back(value);
    }

    pub fn push_back(&self, value: V) {
        let mut state = self.data.borrow_mut();
        state.push_back(value);
    }

    pub fn pop_back(&self) -> Option<V> {
        let mut state = self.data.borrow_mut();
        state.pop_back()
    }

    pub fn take(&self) -> VecDeque<V> {
        let mut state = self.data.borrow_mut();
        std::mem::take(&mut state)
    }

    pub fn replace(&self, child: VecDeque<V>) -> VecDeque<V> {
        let mut state = self.data.borrow_mut();
        std::mem::replace(&mut state, child)
    }

    pub fn is_empty(&self) -> bool {
        let state = self.data.borrow();
        state.is_empty()
    }

    pub fn len(&self) -> usize {
        let state = self.data.borrow();
        state.len()
    }

    pub fn get_mut(&self, index: usize, callback: impl Fn(Option<&mut V>)) {
        let mut state = self.data.borrow_mut();
        let value = state.get_mut(index);
        callback(value);
    }
}

