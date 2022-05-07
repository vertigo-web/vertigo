use std::{
    cmp::PartialEq,
    rc::Rc,
};

use crate::computed::{Client, GraphValue, graph_id::GraphId};

/// A reactive value that is read-only and computed by dependency graph.
///
/// ## Computed directly from Value
///
/// ```rust,no_run
/// use vertigo::{Computed, Value};
///
/// let value = Value::new(5);
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
/// ```rust,no_run
/// use vertigo::{Computed, Value};
///
/// let value = Value::new(2);
///
/// let comp_2 = {
///     let v = value.clone();
///     Computed::from(move || *v.get_value() * 2)
/// };
///
/// assert_eq!(*comp_2.get_value(), 4);
///
/// value.set_value(6);
///
/// assert_eq!(*comp_2.get_value(), 12);
/// ```
pub struct Computed<T: 'static> {
    inner: GraphValue<T>,
}

impl<T: 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, get_value),
        }
    }

    pub fn from<F: Fn() -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, move || {
                Rc::new(get_value())
            })
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id()
    }

    pub fn get_value(&self) -> Rc<T> {
        self.inner.get_value()
    }

    pub fn map_for_render<K: 'static>(self, fun: fn(&Computed<T>) -> K) -> Computed<K> {
        Computed::new(move || {
            let result = fun(&self);
            Rc::new(result)
        })
    }

    pub fn map<K, F: 'static + Fn(&Computed<T>) -> K>(self, fun: F) -> Computed<K> {
        Computed::new(move ||
            Rc::new(fun(&self))
        )
    }

    pub fn id(&self) -> GraphId {
        self.inner.id()
    }
}

impl<T: 'static + PartialEq> Computed<T> {
    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        Client::new(self, call)
    }
}

