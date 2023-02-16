use std::{cmp::PartialEq};
use crate::{
    computed::{Client, GraphValue, graph_id::GraphId},
    dom_value::{render_value, render_value_option},
    dom_list::{render_list, ListRendered},
    dom::{dom_comment_create::DomFragment, dom_node::DomNodeFragment},
    DomElement, Value
};
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
    pub fn from<F: Fn(&Context) -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, move |context| {
                get_value(context)
            })
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id()
    }

    pub fn get(&self, context: &Context) -> T {
        self.inner.get_value(context)
    }

    pub fn map<K: Clone + 'static, F: 'static + Fn(T) -> K>(&self, fun: F) -> Computed<K> {
        Computed::from({
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
    pub fn subscribe<R: 'static, F: Fn(T) -> R + 'static>(self, call: F) -> Client {
        Client::new(self, call)
    }
}

impl<T: 'static + PartialEq + Clone> Computed<T> {
    pub fn render_value<R: Into<DomNodeFragment>>(&self, render: impl Fn(T) -> R + 'static) -> DomFragment {
        render_value(self.clone(), render)
    }

    pub fn render_value_option<R: Into<DomNodeFragment>>(&self, render: impl Fn(T) -> Option<R> + 'static) -> DomFragment {
        render_value_option(self.clone(), render)
    }
}

impl<
    T: PartialEq + Clone + 'static,
    L: IntoIterator<Item=T> + Clone + PartialEq + 'static
> Computed<L> {
    pub fn render_list<
        K: Eq + Hash,
    >(
        &self,
        get_key: impl Fn(&T) -> K + 'static,
        render: impl Fn(&T) -> DomElement + 'static,
    ) -> ListRendered<T> {
        let list = self.map(|inner| {
            inner.into_iter().collect::<Vec<_>>()
        });
        render_list(list, get_key, render)
    }
}

impl<T: Clone + 'static> From<Value<T>> for Computed<T> {
    fn from(val: Value<T>) -> Self {
        val.to_computed()
    }
}

impl<T: Clone + 'static> From<T> for Computed<T> {
    fn from(value: T) -> Self {
        Value::new(value).to_computed()
    }
}

impl<T: Clone + 'static> From<&T> for Computed<T> {
    fn from(value: &T) -> Self {
        Value::new(value.clone()).to_computed()
    }
}

impl From<&str> for Computed<String> {
    fn from(value: &str) -> Self {
        Value::new(value.to_string()).to_computed()
    }
}
