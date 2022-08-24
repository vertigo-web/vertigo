use std::{cmp::PartialEq};
use crate::{computed::{Client, GraphValue, graph_id::GraphId}, dom_value::{render_value, render_value_option}, DomComment, dom_list::render_list, DomNode};
use std::hash::Hash;

use super::context::Context;

/// A reactive value that is read-only and computed by dependency graph.
///
/// ## Computed directly from Value
///
/// ```rust
/// use vertigo::{Computed, Value, transaction};
///
/// let value = Value::new(5);
///
/// let comp = value.to_computed();
///
/// transaction(|context| {
///     assert_eq!(comp.get(context), 5);
/// });
///
/// // Can't do that
/// // comp.set(10);
/// ```
///
/// ## Computed from Value by provided function
///
/// ```rust
/// use vertigo::{Computed, Value, transaction};
///
/// let value = Value::new(2);
///
/// let comp_2 = {
///     let v = value.clone();
///     Computed::from(move |context| v.get(context) * 2)
/// };
///
/// transaction(|context| {
///     assert_eq!(comp_2.get(context), 4);
/// });
///
/// value.set(6);
///
/// transaction(|context| {
///     assert_eq!(comp_2.get(context), 12);
/// });
/// 
/// ```
pub struct Computed<T: Clone> {
    inner: GraphValue<T>,
}

impl<T: Clone + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + 'static> PartialEq for Computed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: Clone + 'static> Computed<T> {
    pub fn new<F: Fn(&Context) -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, move || {
                let context = Context::new();
                get_value(&context)
            }),
        }
    }

    pub fn from<F: Fn(&Context) -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, move || {
                let context = Context::new();
                get_value(&context)
            })
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id()
    }

    pub fn get(&self, _context: &Context) -> T {
        self.inner.get_value(true)
    }

    pub fn map<K: Clone + 'static, F: 'static + Fn(T) -> K>(&self, fun: F) -> Computed<K> {
        Computed::new({
            let computed = self.clone();
            move |context| {
                fun(computed.get(context))
            }
        })
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

impl<T: 'static + PartialEq + Clone> Computed<T> {
    pub fn render_value<R: Into<DomNode>>(&self, render: impl Fn(T) -> R + 'static) -> DomComment {
        render_value(self.clone(), render)
    }

    pub fn render_value_option<R: Into<DomNode>>(&self, render: impl Fn(T) -> Option<R> + 'static) -> DomComment {
        render_value_option(self.clone(), render)
    }
}

impl<
    T: PartialEq + Clone + 'static,
    L: IntoIterator<Item=T> + Clone + PartialEq + 'static
> Computed<L> {
    pub fn render_list<
        K: Eq + Hash,
        R: Into<DomNode>,
    >(
        &self,
        get_key: impl Fn(&T) -> K + 'static,
        render: impl Fn(&T) -> R + 'static,
    ) -> DomComment {
        let list = self.map(|inner| {
            inner.into_iter().collect::<Vec<_>>()
        });
        render_list(list, get_key, render)
    }
}