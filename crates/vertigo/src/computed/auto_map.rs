use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::cmp::PartialEq;

use crate::computed::Computed;
use crate::utils::{
    BoxRefCell,
    EqBox,
};

#[derive(PartialEq)]
pub struct AutoMap<K: Eq + Hash + Clone, V: PartialEq + 'static> {
    create: EqBox<Box<dyn Fn(&K) -> Computed<V>>>,
    values: Rc<EqBox<BoxRefCell<HashMap<K, Computed<V>>>>>,
}

impl<K: Eq + Hash + Clone, V: PartialEq + 'static> AutoMap<K, V> {
    pub fn new<C: Fn(&K) -> Computed<V> + 'static>(create: C) -> AutoMap<K, V> {
        AutoMap {
            create: EqBox::new(Box::new(create)),
            values: Rc::new(EqBox::new(BoxRefCell::new(HashMap::new(), "auto map values"))),
        }
    }

    pub fn get_value(&self, key: &K) -> Computed<V> {
        let item: Option<Computed<V>> = self.values.get_with_context(
            key,
            |state, key| -> Option<Computed<V>> {
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



#[derive(PartialEq, Clone)]
pub struct AutoMapBox<K: Eq + Hash + Clone, V: PartialEq + Clone + 'static> {
    create: Rc<EqBox<Box<dyn Fn(&K) -> V>>>,
    values: Rc<EqBox<BoxRefCell<HashMap<K, V>>>>,
}

impl<K: Eq + Hash + Clone, V: PartialEq + Clone + 'static> AutoMapBox<K, V> {
    pub fn new<C: Fn(&K) -> V + 'static>(create: C) -> AutoMapBox<K, V> {
        AutoMapBox {
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