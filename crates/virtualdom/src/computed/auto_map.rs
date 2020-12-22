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

        self.values.change(
            (key, &(self.create)), 
            |state, (key, create)| -> Computed<V> {
                let item = (*state).get(key);

                if let Some(item) = item {
                    return item.clone();
                }

                let new_item = create(key);

                (*state).insert(key.clone(), new_item.clone());

                new_item
        })
    }
}
