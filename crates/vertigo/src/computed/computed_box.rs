use std::{
    cmp::PartialEq,
    rc::Rc,
};

use crate::computed::{Client, Dependencies, GraphValue, graph_id::GraphId};

/// A reactive value that is read-only and computed by dependency graph.
///
/// ## Computed directly from Value
///
/// ```rust
/// use vertigo::{Computed, Dependencies};
///
/// let deps = Dependencies::default();
///
/// let value = deps.new_value(5);
///
/// let comp = value.to_computed();
///
/// assert_eq!(*comp.get_value(), 5);
///
/// // Can't do that
/// // comp.set_value(10);
/// ```
///
/// ## Computed from Value by provided function
///
/// ```rust
/// use vertigo::{Computed, Dependencies};
///
/// let deps = Dependencies::default();
///
/// let value = deps.new_value(2);
///
/// let comp_2 = {
///     let v = value.clone();
///     deps.from(move || *v.get_value() * 2)
/// };
///
/// assert_eq!(*comp_2.get_value(), 4);
///
/// value.set_value(6);
///
/// assert_eq!(*comp_2.get_value(), 12);
/// ```
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
            inner: GraphValue::new(&deps, true, get_value),
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id()
    }

    pub fn get_value(&self) -> Rc<T> {
        self.inner.get_value()
    }

    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        Client::new(self.inner.deps(), self, call)
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

    pub fn map<K: PartialEq, F: 'static + Fn(&Computed<T>) -> K>(self, fun: F) -> Computed<K> {
        let deps = self.inner.deps();

        Computed::new(deps, move ||
            Rc::new(fun(&self))
        )
    }

    pub fn id(&self) -> GraphId {
        self.inner.id()
    }
}

impl<T: PartialEq + 'static> PartialEq for Computed<T> {
    fn eq(&self, other: &Computed<T>) -> bool {
        self.inner.id() == other.inner.id()
    }
}
