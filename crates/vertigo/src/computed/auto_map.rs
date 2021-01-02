use core::{
    hash::Hash,
    cmp::PartialEq,
};
use alloc::{
    rc::Rc,
    boxed::Box,
    collections::BTreeMap,
};

use crate::computed::Computed;
use crate::utils::{
    BoxRefCell,
    EqBox,
};

#[derive(PartialEq)]
pub struct AutoMap<K: Eq + Hash + Clone + Ord, V: PartialEq + 'static> {
    create: EqBox<Box<dyn Fn(&K) -> Computed<V>>>,
    values: Rc<EqBox<BoxRefCell<BTreeMap<K, Computed<V>>>>>,
}

impl<K: Eq + Hash + Clone + Ord, V: PartialEq + 'static> AutoMap<K, V> {
    pub fn new<C: Fn(&K) -> Computed<V> + 'static>(create: C) -> AutoMap<K, V> {
        AutoMap {
            create: EqBox::new(Box::new(create)),
            values: Rc::new(EqBox::new(BoxRefCell::new(BTreeMap::new()))),
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
