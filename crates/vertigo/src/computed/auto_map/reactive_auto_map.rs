use std::fmt::Debug;
use std::{hash::Hash, rc::Rc};

use crate::{Computed, Value};

use super::{AutoMap, CreateType};

/// A structure similar to [ReactiveAutoMap]
/// but wrapped in [Value] which allows to rectively modify these values or clear the map.
///
/// ```rust
/// use vertigo::{ReactiveAutoMap, transaction};
///
/// let my_map = ReactiveAutoMap::<i32, i32>::new(|_, x| x*2);
///
/// transaction(|context| {
///     assert_eq!(my_map.get(5).get(context), 10);
/// });
/// ```
#[derive(Clone)]
pub struct ReactiveAutoMap<K, V> {
    map: Value<AutoMap<K, V>>,
    create: Rc<CreateType<K, V>>,
}

impl<K, V> Debug for ReactiveAutoMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactiveAutoMap").finish()
    }
}

impl<K, V> PartialEq for ReactiveAutoMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.map.eq(&other.map)
    }
}

impl<K: Eq + Hash + Clone + 'static, V: Clone + 'static> ReactiveAutoMap<K, V> {
    pub fn new<C: Fn(&AutoMap<K, V>, &K) -> V + 'static>(create: C) -> Self {
        let create_rc: Rc<CreateType<K, V>>  = Rc::new(Box::new(create));
        Self {
            map: Value::new(AutoMap::new_from_rc(create_rc.clone())),
            create: create_rc,
        }
    }

    pub fn get(&self, key: impl Into<K>) -> Computed<V> {
        let key = key.into();
        self.map.map(move |map| map.get(&key))
    }

    pub fn clear(&self) {
        self.map.set(AutoMap::new_from_rc(self.create.clone()))
    }
}
