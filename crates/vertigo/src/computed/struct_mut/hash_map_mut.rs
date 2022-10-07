use std::collections::HashMap;
use std::hash::Hash;
use super::inner_value::InnerValue;

pub struct HashMapMut<K, V> {
    data: InnerValue<HashMap<K, V>>,
}

impl<K: Eq + Hash, V> Default for HashMapMut<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash, V> HashMapMut<K, V> {
    pub fn new() -> HashMapMut<K, V> {
        HashMapMut {
            data: InnerValue::new(HashMap::new()),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let state = self.data.get_mut();
        state.insert(key, value)
    }

    pub fn get_and_map<R, F: FnOnce(&V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let state = self.data.get();

        let item = state.get(key);

        if let Some(elem) = item {
            return Some(callback(elem));
        }

        None
    }

    pub fn must_change<R, F: FnOnce(&mut V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let state = self.data.get_mut();

        let item = state.get_mut(key);

        if let Some(elem) = item {
            return Some(callback(elem));
        }

        None
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let state = self.data.get_mut();
        state.remove(key)
    }

    pub fn mem_replace(&self, new_map: HashMap<K, V>) -> HashMap<K, V> {
        let state = self.data.get_mut();
        std::mem::replace(state, new_map)
    }

    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&self, f: F) {
        let state = self.data.get_mut();
        state.retain(f)
    }

    pub fn filter_and_map<R>(&self, map: fn(&V) -> Option<R>) -> Vec<R> {
        let state = self.data.get();
        let mut list = Vec::new();
        for (_, value) in (*state).iter() {
            if let Some(mapped) = map(value) {
                list.push(mapped);
            }
        }

        list
    }
}

impl<K: Eq + Hash, V: Clone> HashMapMut<K, V> {
    pub fn get_all_values(&self) -> Vec<V> {
        let state = self.data.get();

        let mut out = Vec::new();

        for (_, callback) in state.iter() {
            out.push((*callback).clone());
        }

        out
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let state = self.data.get();
        state.get(key).map(|value| (*value).clone())
    }
}

impl<K: Eq + Hash, V: PartialEq> HashMapMut<K, V> {
    pub fn insert_and_check(&self, key: K, value: V) -> bool {
        let state = self.data.get_mut();
        let is_change = state.get(&key) != Some(&value);
        state.insert(key, value);
        is_change
    }

}

