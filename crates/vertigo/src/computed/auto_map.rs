use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::computed::{
    BoxRefCell,
    Dependencies,
    Computed,
    GraphId,
};

pub struct AutoMap<K: Eq + Hash + Clone, V: 'static> {
    id: GraphId,
    create: Box<dyn Fn(&K) -> Computed<V>>,
    values: Rc<BoxRefCell<HashMap<K, Computed<V>>>>,
    deps: Dependencies,
}

impl<K: Eq + Hash + Clone, V: 'static> AutoMap<K, V> {
    pub fn new<C: Fn(&K) -> Computed<V> + 'static>(deps: &Dependencies, create: C) -> AutoMap<K, V> {
        AutoMap {
            id: GraphId::default(),
            create: Box::new(create),
            values: Rc::new(BoxRefCell::new(HashMap::new())),
            deps: deps.clone(),
        }
    }

    pub fn getValue(&self, key: &K) -> Computed<V> {
        self.deps.reportDependenceInStack(self.id.clone());

        let item: Option<Computed<V>> = self.values.getWithContext(
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
