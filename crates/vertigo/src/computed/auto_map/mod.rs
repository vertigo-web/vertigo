use std::fmt::Debug;
use std::{hash::Hash, rc::Rc};

use crate::struct_mut::HashMapMut;

mod reactive_auto_map;
pub use reactive_auto_map::ReactiveAutoMap;

type CreateType<K, V> = Box<dyn Fn(&AutoMap<K, V>, &K) -> V>;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A structure similar to HashMap
/// but allows to provide a function `create` for creating a new value if particular key doesn't exists.
///
/// Such a function can for example [fetch](struct.Driver.html#method.fetch) data from internet.
///
/// ```rust
/// use vertigo::AutoMap;
///
/// let my_map = AutoMap::<i32, i32>::new(|_, x| x*2);
/// assert_eq!(my_map.get(&5), 10);
/// ```
#[derive(Clone)]
pub struct AutoMap<K, V> {
    id: u64,
    create: Rc<CreateType<K, V>>,
    values: Rc<HashMapMut<K, V>>,
}

impl<K, V> Debug for AutoMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoMap").finish()
    }
}

impl<K, V> PartialEq for AutoMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<K: Eq + Hash + Clone, V: Clone> AutoMap<K, V> {
    pub fn new<C: Fn(&Self, &K) -> V + 'static>(create: C) -> Self {
        Self {
            id: get_unique_id(),
            create: Rc::new(Box::new(create)),
            values: Rc::new(HashMapMut::new()),
        }
    }

    pub fn new_from_rc(create: Rc<CreateType<K, V>>) -> Self {
        Self {
            id: get_unique_id(),
            create,
            values: Rc::new(HashMapMut::new()),
        }
    }

    pub fn get(&self, key: &K) -> V {
        let item: Option<V> = self.values.get(key);

        if let Some(item) = item {
            return item;
        }

        let new_item = {
            let create = &self.create;
            create(self, key)
        };

        self.values.insert(key.clone(), new_item.clone());

        new_item
    }
}
