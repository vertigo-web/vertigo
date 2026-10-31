use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    hash::{BuildHasher, BuildHasherDefault, Hash},
};

use crate::fast_hash::{FastBuildHasher, FastHasher};

/// The values of a [`#[store]`](crate::store) function with arguments, one per set of them.
///
/// Each value is kept together with its arguments behind `dyn Any`, under the hash of the
/// arguments. A map generic over the arguments and the value would bring its own copy of the
/// hash map code into the wasm for every store, and one more for each further argument. This
/// one is the same type for all of them: what a store adds is only how to hash, compare and
/// clone its own types.
///
/// Nothing is ever removed, so arguments whose hash is already taken by others go under the
/// next free one, and a lookup walks on until it finds them or a free hash.
pub struct StoreMap {
    entries: RefCell<HashMap<u64, Box<dyn Any>, FastBuildHasher>>,
}

impl Default for StoreMap {
    fn default() -> Self {
        Self::new()
    }
}

impl StoreMap {
    pub const fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::with_hasher(BuildHasherDefault::new())),
        }
    }

    pub fn hash(key: &impl Hash) -> u64 {
        BuildHasherDefault::<FastHasher>::new().hash_one(key)
    }

    /// The value for the arguments `is_key` accepts, or a new one from `create`, kept with the
    /// arguments from `key`. `hash` is [`StoreMap::hash`] of the arguments.
    pub fn get_or_create<K: 'static, V: Clone + 'static>(
        &self,
        hash: u64,
        is_key: impl Fn(&K) -> bool,
        key: impl FnOnce() -> K,
        create: impl FnOnce() -> V,
    ) -> V {
        let mut found = None;
        self.find(hash, &mut |entry| match entry.downcast_ref::<(K, V)>() {
            Some((stored, value)) if is_key(stored) => {
                found = Some(value.clone());
                true
            }
            _ => false,
        });

        if let Some(value) = found {
            return value;
        }

        // Not borrowed while creating: the body may well ask the same store for other arguments.
        let value = create();
        self.insert(hash, Box::new((key(), value.clone())));
        value
    }

    fn find(&self, mut hash: u64, take: &mut dyn FnMut(&dyn Any) -> bool) {
        let entries = self.entries.borrow();
        while let Some(entry) = entries.get(&hash) {
            if take(&**entry) {
                return;
            }
            hash = hash.wrapping_add(1);
        }
    }

    fn insert(&self, mut hash: u64, entry: Box<dyn Any>) {
        let mut entries = self.entries.borrow_mut();
        while entries.contains_key(&hash) {
            hash = hash.wrapping_add(1);
        }
        entries.insert(hash, entry);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    fn get(map: &StoreMap, hash: u64, name: &str, created: &Cell<u32>) -> String {
        map.get_or_create(
            hash,
            |stored: &(String,)| stored.0 == name,
            || (name.to_string(),),
            || {
                created.set(created.get() + 1);
                format!("value of {name}")
            },
        )
    }

    #[test]
    fn creates_once_per_key() {
        let map = StoreMap::new();
        let created = Cell::new(0);

        for name in ["a", "b", "a", "b", "a"] {
            let value = get(&map, StoreMap::hash(&(name,)), name, &created);
            assert_eq!(value, format!("value of {name}"));
        }
        assert_eq!(created.get(), 2);
    }

    #[test]
    fn keys_sharing_a_hash_stay_apart() {
        let map = StoreMap::new();
        let created = Cell::new(0);

        for name in ["a", "b", "c", "b", "a", "c"] {
            assert_eq!(get(&map, 7, name, &created), format!("value of {name}"));
        }
        assert_eq!(created.get(), 3);

        // the walk goes past a hash taken by another key's overflow too
        assert_eq!(get(&map, 8, "d", &created), "value of d");
        assert_eq!(get(&map, 7, "c", &created), "value of c");
        assert_eq!(created.get(), 4);
    }

    #[test]
    fn body_may_ask_for_other_arguments() {
        let map = StoreMap::new();

        fn depth(map: &StoreMap, n: u32) -> u32 {
            map.get_or_create(
                StoreMap::hash(&(n,)),
                |stored: &(u32,)| stored.0 == n,
                || (n,),
                || match n {
                    0 => 0,
                    _ => depth(map, n - 1) + 1,
                },
            )
        }

        assert_eq!(depth(&map, 5), 5);
        assert_eq!(map.entries.borrow().len(), 6);
    }
}
