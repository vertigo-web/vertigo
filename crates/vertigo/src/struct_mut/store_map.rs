use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    hash::{BuildHasher, BuildHasherDefault, Hash},
    marker::PhantomData,
};

use crate::fast_hash::{FastBuildHasher, FastHasher};

/// The values `V` of a [`#[store]`](crate::store) function with arguments `K`, one per set of
/// them.
///
/// Public only because `#[store]` expands to it.
///
/// Each value is kept together with its arguments behind `dyn Any`, under the hash of the
/// arguments. A map generic over the arguments and the value would bring its own copy of the
/// hash map code into the wasm for every store, and one more for each further argument. This
/// one keeps the types only in its signature: the entries are the same type for all stores,
/// and what a store adds is only how to hash, compare and clone its own types.
pub struct StoreMap<K, V> {
    entries: Entries,
    _types: PhantomData<(K, V)>,
}

impl<K, V> Default for StoreMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> StoreMap<K, V> {
    pub const fn new() -> Self {
        Self {
            entries: Entries(RefCell::new(
                HashMap::with_hasher(BuildHasherDefault::new()),
            )),
            _types: PhantomData,
        }
    }
}

impl<K: 'static, V: Clone + 'static> StoreMap<K, V> {
    /// The value for `arguments`, or a new one from `create`. `is_key` tells the arguments
    /// apart from others with the same hash, and `key` makes the copy kept with the value.
    pub fn get_or_create(
        &self,
        arguments: &impl Hash,
        is_key: impl Fn(&K) -> bool,
        key: impl FnOnce() -> K,
        create: impl FnOnce() -> V,
    ) -> V {
        let hash = BuildHasherDefault::<FastHasher>::new().hash_one(arguments);
        self.get_or_create_hashed(hash, is_key, key, create)
    }

    fn get_or_create_hashed(
        &self,
        hash: u64,
        is_key: impl Fn(&K) -> bool,
        key: impl FnOnce() -> K,
        create: impl FnOnce() -> V,
    ) -> V {
        let mut found = None;
        self.entries
            .find(hash, &mut |entry| match entry.downcast_ref::<(K, V)>() {
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
        self.entries.insert(hash, Box::new((key(), value.clone())));
        value
    }
}

/// The part of [`StoreMap`] that does not depend on its types, so that the wasm has it once.
///
/// Nothing is ever removed, so arguments whose hash is already taken by others go under the
/// next free one, and a lookup walks on until it finds them or a free hash.
struct Entries(RefCell<HashMap<u64, Box<dyn Any>, FastBuildHasher>>);

impl Entries {
    fn find(&self, mut hash: u64, take: &mut dyn FnMut(&dyn Any) -> bool) {
        let entries = self.0.borrow();
        while let Some(entry) = entries.get(&hash) {
            if take(&**entry) {
                return;
            }
            hash = hash.wrapping_add(1);
        }
    }

    fn insert(&self, mut hash: u64, entry: Box<dyn Any>) {
        let mut entries = self.0.borrow_mut();
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

    type Names = StoreMap<(String,), String>;

    fn get(map: &Names, hash: u64, name: &str, created: &Cell<u32>) -> String {
        map.get_or_create_hashed(
            hash,
            |stored| stored.0 == name,
            || (name.to_string(),),
            || {
                created.set(created.get() + 1);
                format!("value of {name}")
            },
        )
    }

    #[test]
    fn creates_once_per_key() {
        let map = Names::new();
        let created = Cell::new(0);

        for name in ["a", "b", "a", "b", "a"] {
            let value = map.get_or_create(
                &(name,),
                |stored| stored.0 == name,
                || (name.to_string(),),
                || {
                    created.set(created.get() + 1);
                    format!("value of {name}")
                },
            );
            assert_eq!(value, format!("value of {name}"));
        }
        assert_eq!(created.get(), 2);
    }

    #[test]
    fn keys_sharing_a_hash_stay_apart() {
        let map = Names::new();
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
        let map = StoreMap::<(u32,), u32>::new();

        fn depth(map: &StoreMap<(u32,), u32>, n: u32) -> u32 {
            map.get_or_create(
                &(n,),
                |stored| stored.0 == n,
                || (n,),
                || match n {
                    0 => 0,
                    _ => depth(map, n - 1) + 1,
                },
            )
        }

        assert_eq!(depth(&map, 5), 5);
        assert_eq!(map.entries.0.borrow().len(), 6);
    }
}
