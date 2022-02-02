use std::{cell::RefCell, collections::BTreeMap};


pub struct BTreeMapMut<K: Ord, V> {
    data: RefCell<BTreeMap<K, V>>,
}

impl<K: Ord, V> Default for BTreeMapMut<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> BTreeMapMut<K, V> {
    pub fn new() -> BTreeMapMut<K, V> {
        BTreeMapMut {
            data: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut state = self.data.borrow_mut();
        state.insert(key, value)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut state = self.data.borrow_mut();
        state.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let state = self.data.borrow();
        state.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        let state = self.data.borrow();
        state.is_empty()
    }

    pub fn take(&self) -> BTreeMap<K, V> {
        let mut state = self.data.borrow_mut();
        std::mem::take(&mut state)
    }

    pub fn map<R>(&self, map_f: impl FnOnce(&BTreeMap<K, V>) -> R) -> R {
        let state = self.data.borrow();
        map_f(&state)
    }

    pub fn change(&self, change_f: impl Fn(&mut BTreeMap<K, V>)) {
        let mut state = self.data.borrow_mut();
        change_f(&mut state)
    }

    pub fn map_and_change<R>(&self, change_f: impl FnOnce(&mut BTreeMap<K, V>) -> R) -> R {
        let mut state = self.data.borrow_mut();
        change_f(&mut state)
    }

    pub fn get_mut<R, F: FnOnce(&mut V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let mut state = self.data.borrow_mut();

        let item = state.get_mut(key);

        if let Some(elem) = item {
            return Some(callback(elem));
        }

        None
    }

}

impl<K: Ord, V: Clone> BTreeMapMut<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        let state = self.data.borrow();
        state.get(key).cloned()
    }
}

