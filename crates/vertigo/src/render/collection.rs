use crate::computed::ValueSynchronize;
use crate::{Computed, Value, dev::HashMapMut, transaction};
use std::{collections::HashSet, hash::Hash, marker::PhantomData, rc::Rc};

use log;

pub trait CollectionKey {
    type Key: Eq + Hash + Clone + std::fmt::Debug + 'static;
    type Value: Clone + PartialEq + 'static;
    fn get_key(val: &Self::Value) -> Self::Key;
}

#[derive(Clone)]
struct ItemData<V: Clone + PartialEq + 'static> {
    value: Value<V>,
    computed: Computed<V>,
}

struct ItemDataCollection<T: CollectionKey + 'static> {
    items: Rc<HashMapMut<T::Key, ItemData<T::Value>>>,
    _marker: PhantomData<T>,
}

impl<T: CollectionKey + 'static> Clone for ItemDataCollection<T> {
    fn clone(&self) -> Self {
        ItemDataCollection {
            items: self.items.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: CollectionKey + 'static> ItemDataCollection<T> {
    pub fn new() -> ItemDataCollection<T> {
        ItemDataCollection {
            items: Rc::new(HashMapMut::new()),
            _marker: PhantomData,
        }
    }

    fn get_item(&self, key: &T::Key, item: &T::Value) -> CollectionModel<T> {
        if let Some(model) = self.items.get(key) {
            model.value.set(item.clone());
            return CollectionModel {
                key: key.clone(),
                model: model.computed,
            };
        }

        let model_value = Value::new(item.clone());
        let model_computed = model_value.to_computed();

        let model = ItemData {
            value: model_value,
            computed: model_computed,
        };

        self.items.insert(key.clone(), model.clone());

        CollectionModel {
            key: key.clone(),
            model: model.computed,
        }
    }

    fn translate(&self, list: Rc<Vec<T::Value>>) -> Vec<CollectionModel<T>> {
        let mut new_order: Vec<CollectionModel<T>> = Vec::with_capacity(list.len());
        let mut seen_keys = HashSet::new();

        for item in list.as_ref() {
            let key = T::get_key(item);

            if seen_keys.contains(&key) {
                log::error!("Duplicate key found in Collection: {:?}", key);
                continue;
            }

            seen_keys.insert(key.clone());

            let model = self.get_item(&key, item);
            new_order.push(model);
        }

        new_order
    }

    fn retain(&self, new_keys: HashSet<T::Key>) {
        self.items.retain(|k, _| new_keys.contains(k));
    }
}

pub struct CollectionModel<T: CollectionKey + 'static> {
    pub key: T::Key,
    pub model: Computed<T::Value>,
}

impl<T: CollectionKey + 'static> Clone for CollectionModel<T> {
    fn clone(&self) -> Self {
        CollectionModel {
            key: self.key.clone(),
            model: self.model.clone(),
        }
    }
}

impl<T: CollectionKey + 'static> PartialEq for CollectionModel<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.model == other.model
    }
}

pub struct Collection<T: CollectionKey + 'static> {
    items: ItemDataCollection<T>,
    order: Value<Vec<CollectionModel<T>>>,
}

impl<T: CollectionKey + 'static> Clone for Collection<T> {
    fn clone(&self) -> Self {
        Collection {
            items: self.items.clone(),
            order: self.order.clone(),
        }
    }
}

impl<T: CollectionKey + 'static> Collection<T> {
    pub fn new(list: Rc<Vec<T::Value>>) -> Collection<T> {
        let items = ItemDataCollection::new();
        let order = items.translate(list);

        Collection {
            items,
            order: Value::new(order),
        }
    }

    pub fn set(&self, list: Rc<Vec<T::Value>>) {
        transaction(|_ctx| {
            let new_order = self.items.translate(list);

            let new_keys = new_order
                .iter()
                .map(|item| item.key.clone())
                .collect::<HashSet<T::Key>>();

            self.order.set(new_order);

            self.items.retain(new_keys);
        })
    }

    pub fn get(&self) -> Computed<Vec<CollectionModel<T>>> {
        self.order.to_computed()
    }
}

impl<T: CollectionKey + 'static> ValueSynchronize<Rc<Vec<T::Value>>> for Collection<T> {
    fn new(value: Rc<Vec<T::Value>>) -> Self {
        Collection::new(value)
    }

    fn set(&self, value: Rc<Vec<T::Value>>) {
        self.set(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_basic() {
        #[derive(Clone, PartialEq, Debug)]
        struct Item {
            id: i32,
            name: String,
        }

        struct ItemId;
        impl CollectionKey for ItemId {
            type Key = i32;
            type Value = Item;
            fn get_key(val: &Item) -> i32 {
                val.id
            }
        }

        let col = Collection::<ItemId>::new(Rc::new(Vec::new()));

        let items = vec![
            Item {
                id: 1,
                name: "One".into(),
            },
            Item {
                id: 2,
                name: "Two".into(),
            },
        ];

        col.set(Rc::new(items.clone()));

        transaction(|ctx| {
            let res = col.get().get(ctx);
            assert_eq!(res.len(), 2);
            assert_eq!(res[0].model.get(ctx).name, "One");
            assert_eq!(res[1].model.get(ctx).name, "Two");
        });
    }

    #[test]
    fn test_collection_reactivity() {
        #[derive(Clone, PartialEq, Debug)]
        struct Item {
            id: i32,
            val: i32,
        }

        struct ItemId;
        impl CollectionKey for ItemId {
            type Key = i32;
            type Value = Item;
            fn get_key(val: &Item) -> i32 {
                val.id
            }
        }

        let col = Collection::<ItemId>::new(Rc::new(Vec::new()));

        col.set(Rc::new(vec![Item { id: 1, val: 10 }]));

        let list_computed = col.get();
        let item_computed = transaction(|ctx| {
            // list[0] is (Key, Computed<V>)
            list_computed.get(ctx)[0].model.clone()
        });

        transaction(|ctx| {
            assert_eq!(item_computed.get(ctx).val, 10);
        });

        // Update item 1
        col.set(Rc::new(vec![Item { id: 1, val: 20 }]));

        transaction(|ctx| {
            assert_eq!(item_computed.get(ctx).val, 20);
        });

        // Add item
        col.set(Rc::new(vec![
            Item { id: 1, val: 20 },
            Item { id: 2, val: 30 },
        ]));

        transaction(|ctx| {
            let new_list = list_computed.get(ctx);
            assert_eq!(new_list.len(), 2);
            // Ensure old reference still works and has correct value
            assert_eq!(item_computed.get(ctx).val, 20);
        });
    }
}
