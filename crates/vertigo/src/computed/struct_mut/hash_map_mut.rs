use std::{collections::HashMap, fmt::Display, hash::Hash};
use std::cell::RefCell;

pub struct HashMapMut<K: Eq + Hash + Display, V> {
    data: RefCell<HashMap<K, V>>,
}

impl<K: Eq + Hash + Display, V> Default for HashMapMut<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Display, V> HashMapMut<K, V> {
    pub fn new() -> HashMapMut<K, V> {
        HashMapMut {
            data: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut state = self.data.borrow_mut();
        state.insert(key, value)
    }

    pub fn get_and_map<R, F: FnOnce(&V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let state = self.data.borrow();

        let item = state.get(key);

        if let Some(elem) = item {
            return Some(callback(elem));
        }

        None
    }

    pub fn must_change<R, F: FnOnce(&mut V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let mut state = self.data.borrow_mut();

        let item = state.get_mut(key);

        if let Some(elem) = item {
            return Some(callback(elem));
        }

        None
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut state = self.data.borrow_mut();
        state.remove(key)
    }

    pub fn mem_replace(&self, new_map: HashMap<K, V>) -> HashMap<K, V> {
        let mut state = self.data.borrow_mut();
        std::mem::replace(&mut state, new_map)
    }

    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&self, f: F) {
        let mut state = self.data.borrow_mut();
        state.retain(f)
    }
}

impl<K: Eq + Hash + Display, V: Clone> HashMapMut<K, V> {
    pub fn get_all_values(&self) -> Vec<V> {
        let state = self.data.borrow();

        let mut out = Vec::new();

        for (_, callback) in state.iter() {
            out.push((*callback).clone());
        }

        out
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let state = self.data.borrow();
        state.get(key).map(|value| (*value).clone())
    }
}

impl<K: Eq + Hash + Display, V: PartialEq> HashMapMut<K, V> {
    pub fn insert_and_check(&self, key: K, value: V) -> bool {
        let mut state = self.data.borrow_mut();
        let is_change = state.get(&key) != Some(&value);
        state.insert(key, value);
        is_change
    }

}

