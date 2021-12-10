use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::cmp::PartialEq;

use crate::utils::{
    BoxRefCell,
    EqBox,
};


type CreateType<K, V> = EqBox<Box<dyn Fn(&K) -> V>>;

/// A structure similar to HashMap
/// but allows to provide a function `create` for creating a new value if particular key doesn't exists.
///
/// Such a function can for example [fetch](struct.Driver.html#method.fetch) data from internet.
///
/// ```rust
/// use vertigo::AutoMap;
///
/// let my_map = AutoMap::<i32, i32>::new(|x| x*2);
/// assert_eq!(my_map.get_value(&5), 10);
/// ```
#[derive(PartialEq, Clone)]
pub struct AutoMap<K: Eq + Hash + Clone, V: PartialEq + Clone + 'static> {
    create: Rc<CreateType<K, V>>,
    values: Rc<EqBox<BoxRefCell<HashMap<K, V>>>>,
}

impl<K: Eq + Hash + Clone, V: PartialEq + Clone + 'static> AutoMap<K, V> {
    pub fn new<C: Fn(&K) -> V + 'static>(create: C) -> AutoMap<K, V> {
        AutoMap {
            create: Rc::new(EqBox::new(Box::new(create))),
            values: Rc::new(EqBox::new(BoxRefCell::new(HashMap::new(), "auto map box values"))),
        }
    }

    pub fn get_value(&self, key: &K) -> V {
        let item: Option<V> = self.values.get_with_context(
            key,
            |state, key| -> Option<V> {
                let item = (*state).get(key);

                if let Some(item) = item {
                    return Some(item.clone());
                }

                None
            }
        );

        if let Some(item) = item {
            return item;
        }

        let new_item = {
            let create = &self.create;
            create(key)
        };

        self.values.change(
            (key, &new_item),
            |state, (key, new_item)| {
                (*state).insert(key.clone(), new_item.clone());
            }
        );

        new_item
    }
}