use std::hash::Hash;
use std::rc::Rc;

use crate::{
    computed::{graph_id::GraphId, GraphValue},
    render::{render_list, render_value, render_value_option},
    struct_mut::ValueMut,
    DomNode, DropResource, Value,
};

use super::context::Context;

/// A reactive value that is read-only and computed by dependency graph.
///
/// ## Computed directly from Value
///
/// ```rust
/// use vertigo::{Value, transaction};
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
    inner: Rc<GraphValue<T>>,
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
            inner: GraphValue::new(true, move |context| get_value(context)),
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
            move |context| fun(computed.get(context))
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id()
    }
}

impl<T: 'static + PartialEq + Clone> Computed<T> {
    pub fn subscribe<R: 'static, F: Fn(T) -> R + 'static>(self, call: F) -> DropResource {
        let prev_value = ValueMut::new(None);

        let resource_box = ValueMut::new(None);

        let graph_value = GraphValue::new(false, move |context| {
            let value = self.get(context);

            let should_update = prev_value.set_if_changed(Some(value.clone()));

            if should_update {
                let resource = call(value);
                resource_box.change(move |inner| {
                    *inner = Some(resource);
                });
            }
        });

        let context = Context::new();
        graph_value.get_value(&context);
        let _ = context;

        DropResource::from_struct(graph_value)
    }
}

impl<T: 'static + Clone> Computed<T> {
    pub fn subscribe_all<R: 'static, F: Fn(T) -> R + 'static>(self, call: F) -> DropResource {
        let resource_box = ValueMut::new(None);

        let graph_value = GraphValue::new(false, move |context| {
            let value = self.get(context);

            let resource = call(value);
            resource_box.change(move |inner| {
                *inner = Some(resource);
            });
        });

        let context = Context::new();
        graph_value.get_value(&context);
        let _ = context;

        DropResource::from_struct(graph_value)
    }
}

impl<T: 'static + PartialEq + Clone> Computed<T> {
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        render_value(self.clone(), render)
    }

    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        render_value_option(self.clone(), render)
    }
}

impl<T: PartialEq + Clone + 'static, L: IntoIterator<Item = T> + Clone + PartialEq + 'static>
    Computed<L>
{
    pub fn render_list<K: Eq + Hash>(
        &self,
        get_key: impl Fn(&T) -> K + 'static,
        render: impl Fn(&T) -> DomNode + 'static,
    ) -> DomNode {
        let list = self.map(|inner| inner.into_iter().collect::<Vec<_>>());
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
