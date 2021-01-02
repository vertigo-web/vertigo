use std::rc::Rc;
use std::cmp::PartialEq;

use crate::{
    computed::{
        Dependencies,
        Client,
        graph_id::GraphId,
        GraphValue
    }
};

pub struct Computed<T: PartialEq + 'static> {
    inner: GraphValue<T>,
}

impl<T: PartialEq + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone(),
        }
    }
}

impl<T: PartialEq + 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Dependencies, get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new_computed(&deps, get_value)
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id()
    }

    pub fn get_value(&self) -> Rc<T> {
        self.inner.get_value(true)
    }

    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        Client::new(self.inner.deps(), self.clone(), call)
    }

    pub fn dependencies(&self) -> Dependencies {
        self.inner.deps()
    }

    pub fn map_for_render<K: PartialEq>(self, fun: fn(&Computed<T>) -> K) -> Computed<K> {
        let deps = self.inner.deps();

        Computed::new(deps, move || {
            let result = fun(&self);
            Rc::new(result)
        })
    }

    pub fn map<K: PartialEq, F: 'static + Fn(&Computed<T>) -> Rc<K>>(self, fun: F) -> Computed<K> {
        let deps = self.inner.deps();

        Computed::new(deps, move || {
            let result = fun(&self);
            result
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id()
    }
}

impl<T: PartialEq + 'static> PartialEq for Computed<T> {
    fn eq(&self, other: &Computed<T>) -> bool {
        self.inner.id() == other.inner.id()
    }

    fn ne(&self, other: &Computed<T>) -> bool {
        self.inner.id() != other.inner.id()
    }
}
