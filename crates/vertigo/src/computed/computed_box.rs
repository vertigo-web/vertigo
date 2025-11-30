use std::hash::Hash;
use std::rc::Rc;

use crate::{
    render::{render_list, render_value, render_value_option},
    DomNode,
};

use super::{
    context::Context, graph_id::GraphId, struct_mut::ValueMut, DropResource, GraphValue, Value,
};

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
    /// Creates new [`Computed<T>`] which state is determined by provided generator function.
    pub fn from<F: Fn(&Context) -> T + 'static>(get_value: F) -> Computed<T> {
        Computed {
            inner: GraphValue::new(true, move |context| get_value(context)),
        }
    }

    /// Get current value, it will be computed on-the-fly if the previous one ran out of date.
    pub fn get(&self, context: &Context) -> T {
        self.inner.get_value(context)
    }

    /// Reactively convert [`Computed<T>`] to [`Computed<K>`] with provided transformation function applied.
    pub fn map<K: Clone + 'static, F: 'static + Fn(T) -> K>(&self, fun: F) -> Computed<K> {
        Computed::from({
            let myself = self.clone();
            move |context| fun(myself.get(context))
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id()
    }

    /// Do something every time the value inside [Computed] is triggered.
    ///
    /// Note that the `callback` is fired every time the value in [Computed] is computed, even if the outcome value is not changed.
    pub fn subscribe_all<R: 'static, F: Fn(T) -> R + 'static>(self, callback: F) -> DropResource {
        let resource_box = ValueMut::new(None);

        let graph_value = GraphValue::new(false, move |context| {
            let value = self.get(context);

            let resource = callback(value);
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

impl<T: Clone + PartialEq + 'static> Computed<T> {
    /// Do something every time the value inside [Computed] is changed.
    ///
    /// Note that the `callback` is fired only if the value *really* changes. This means that even if computation takes place with different source value,
    /// but the resulting value is the same as old one, the `callback` is not fired.
    pub fn subscribe<R: 'static, F: Fn(T) -> R + 'static>(self, callback: F) -> DropResource {
        let prev_value = ValueMut::new(None);

        let resource_box = ValueMut::new(None);

        let graph_value = GraphValue::new(false, move |context| {
            let value = self.get(context);

            let should_update = prev_value.set_if_changed(Some(value.clone()));

            if should_update {
                let resource = callback(value);
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

    /// Render value inside this [Computed]. See [Value::render_value()] for examples.
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        render_value(self.clone(), render)
    }

    /// Render optional value inside this [Computed]. See [Value::render_value_option()] for examples.
    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        render_value_option(self.clone(), render)
    }
}

impl<T: Clone + PartialEq + 'static, L: IntoIterator<Item = T> + Clone + 'static> Computed<L> {
    /// Render iterable value inside this [Computed]. See [Value::render_list()] for examples.
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
