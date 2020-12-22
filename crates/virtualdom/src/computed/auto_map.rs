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
    pub fn new(deps: Dependencies, create: Box<dyn Fn(&K) -> Computed<V>>) -> AutoMap<K, V> {
        AutoMap {
            id: GraphId::default(),
            create,
            values: Rc::new(BoxRefCell::new(HashMap::new())),
            deps
        }
    }

    pub fn getValue(&self, key: &K) -> Computed<V> {
        self.deps.reportDependenceInStack(self.id.clone());

        let values_inner = self.values.get(|state| state.clone());

        match values_inner.get(key) {
            None => {
                let new_val = (self.create)(key);
                self.values.change(new_val.clone(), |state, value| {
                    state.insert(key.to_owned(), value);
                });
                new_val
            }
            Some(v) => v.clone(),
        }
    }
}
