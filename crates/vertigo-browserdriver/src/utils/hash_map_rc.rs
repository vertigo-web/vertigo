use std::{collections::HashMap, fmt::Display, hash::Hash, rc::Rc};
use vertigo::utils::BoxRefCell;

#[derive(Clone)]
pub struct HashMapRc<K: Eq + Hash + Display, V> {
    label: &'static str,
    data: Rc<BoxRefCell<HashMap<K, V>>>,
}

impl<K: Eq + Hash + Display, V> HashMapRc<K, V> {
    pub fn new(label: &'static str) -> HashMapRc<K, V> {
        HashMapRc {
            label,
            data: Rc::new(BoxRefCell::new(HashMap::new(), "HashMapRc")),
        }
    }

    pub fn insert(&self, key: K, value: V) {
        self.data.change((key, value), |state, (key, value)| {
            state.insert(key, value);
        });
    }

    pub fn must_get<R, F: FnOnce(&V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        self.data
            .get_with_context((self.label, key, callback), |state, (label, key, callback)| {
                let item = state.get(key);

                if let Some(elem) = item {
                    return Some(callback(elem));
                }

                log::error!("{} -> get -> Missing element with id={}", label, key);
                None
            })
    }

    pub fn must_change<R, F: FnOnce(&mut V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        self.data
            .change((self.label, key, callback), |state, (label, key, callback)| {
                let item = state.get_mut(key);

                if let Some(elem) = item {
                    return Some(callback(elem));
                }

                log::error!("{} ->change ->  Missing element with id={}", label, key);
                None
            })
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.change(key, |state, key| {
            state.remove(key)
        })
    }
}

impl<K: Eq + Hash + Display, V: Clone> HashMapRc<K, V> {
    pub fn get_all_values(&self) -> Vec<V> {
        self.data.get(|state| {
            let mut out = Vec::new();

            for (_, callback) in state.iter() {
                out.push((*callback).clone());
            }

            out
        })
    }

    pub fn must_get_clone(&self, key: &K) -> Option<V> {
        self.data.get_with_context(key, |state, key| {
            state.get(key).map(|value| (*value).clone())
        })
    }
}
