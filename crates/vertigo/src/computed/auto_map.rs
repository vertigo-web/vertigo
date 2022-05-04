use std::{
    hash::Hash,
    rc::Rc,
    fmt::Display,
};

use crate::{struct_mut::HashMapMut};

type CreateType<K, V> = Box<dyn Fn(&K) -> V>;

/// A structure similar to HashMap
/// but allows to provide a function `create` for creating a new value if particular key doesn't exists.
///
/// Such a function can for example [fetch](struct.Driver.html#method.fetch) data from internet.
///
/// ```rust
/// use vertigo::AutoMap;
///
/// let my_map = AutoMap::<i32, i32>::new(|x| x*2);
/// assert_eq!(my_map.get(&5), 10);
/// ```
#[derive(Clone)]
pub struct AutoMap<K: Eq + Hash + Clone + Display, V: Clone + 'static> {
    create: Rc<CreateType<K, V>>,
    values: Rc<HashMapMut<K, V>>,
}

impl<K: Eq + Hash + Clone + Display, V: Clone + 'static> AutoMap<K, V> {
    pub fn new<C: Fn(&K) -> V + 'static>(create: C) -> AutoMap<K, V> {
        AutoMap {
            create: Rc::new(Box::new(create)),
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
            create(key)
        };

        self.values.insert(key.clone(), new_item.clone());

        new_item
    }
}
