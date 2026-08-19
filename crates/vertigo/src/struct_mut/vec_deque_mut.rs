use std::collections::VecDeque;

use super::inner_value::InnerValue;

pub struct VecDequeMut<V> {
    data: InnerValue<VecDeque<V>>,
}

impl<V> Default for VecDequeMut<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> VecDequeMut<V> {
    pub fn new() -> VecDequeMut<V> {
        VecDequeMut {
            data: InnerValue::new(VecDeque::new()),
        }
    }

    pub fn push(&self, value: V) {
        let state = self.data.get_mut();
        state.push_back(value);
    }

    pub fn push_back(&self, value: V) {
        let state = self.data.get_mut();
        state.push_back(value);
    }

    pub fn pop_back(&self) -> Option<V> {
        let state = self.data.get_mut();
        state.pop_back()
    }

    pub fn take(&self) -> VecDeque<V> {
        let state = self.data.get_mut();
        std::mem::take(state)
    }

    pub fn replace(&self, child: VecDeque<V>) -> VecDeque<V> {
        let state = self.data.get_mut();
        std::mem::replace(state, child)
    }

    pub fn is_empty(&self) -> bool {
        let state = self.data.get();
        state.is_empty()
    }

    pub fn len(&self) -> usize {
        let state = self.data.get();
        state.len()
    }

    pub fn get_mut(&self, index: usize, callback: impl Fn(Option<&mut V>)) {
        let state = self.data.get_mut();
        let value = state.get_mut(index);
        callback(value);
    }
}
