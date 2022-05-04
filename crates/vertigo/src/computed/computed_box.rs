use std::{
    cmp::PartialEq,
};

use crate::computed::{Client, GraphValue, graph_id::GraphId};

/// A reactive value that is read-only and computed by dependency graph.
///
/// ## Computed directly from Value
///
/// ```rust
/// use vertigo::{Computed, Value};
///
/// let value = Value::new(5);
///
/// let comp = value.to_computed();
///
/// assert_eq!(comp.get(), 5);
///
/// // Can't do that
/// // comp.set(10);
/// ```
///
/// ## Computed from Value by provided function
///
/// ```rust
/// use vertigo::{Computed, Value};
///
/// let value = Value::new(2);
///
/// let comp_2 = {
///     let v = value.clone();
///     Computed::from(move || v.get() * 2)
/// };
///
/// assert_eq!(comp_2.get(), 4);
///
/// value.set(6);
///
/// assert_eq!(comp_2.get(), 12);
/// ```
pub struct Computed<T: Clone + 'static> {
    inner: GraphValue<T>,
}

impl<T: Clone + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + 'static> Computed<T> {
    pub fn new<F: Fn() -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, get_value),
        }
    }

    pub fn from<F: Fn() -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, move || {
                get_value()
            })
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id()
    }

    pub fn get(&self) -> T {
        self.inner.get_value()
    }

    pub fn map_for_render<K: Clone + 'static>(self, fun: fn(&Computed<T>) -> K) -> Computed<K> {
        Computed::new(move ||
            fun(&self)
        )
    }

    pub fn map<K: Clone, F: 'static + Fn(&Computed<T>) -> K>(self, fun: F) -> Computed<K> {
        Computed::new(move ||
            fun(&self)
        )
    }

    pub fn id(&self) -> GraphId {
        self.inner.id()
    }
}

impl<T: 'static + PartialEq + Clone> Computed<T> {
    pub fn subscribe<F: Fn(T) + 'static>(self, call: F) -> Client {
        Client::new(self, call)
    }
}

