use std::rc::Rc;

use crate::{
    DomNode,
    render::{render_value, render_value_option},
};

use super::{
    DropResource, GraphValue, Value, context::Context, get_dependencies, graph_id::GraphId,
    struct_mut::ValueMut,
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

    /// Executes a closure when the `Computed` value starts being observed.
    ///
    /// The provided closure should return a [`DropResource`] which will be dropped
    /// when the value stops being observed. This is especially useful for integrating
    /// with sources that are not reactive (like external data fetching or socket connections),
    /// allowing side effects to start only when they are actually needed by the UI.
    pub fn when_connect<F: Fn() -> DropResource + 'static>(&self, create: F) -> Computed<T> {
        let new_computed = Computed::from({
            let parent = self.clone();
            move |context| parent.get(context)
        });

        get_dependencies()
            .graph
            .external_connections
            .register_connect(new_computed.id(), Rc::new(create));

        new_computed
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

        let context = Context::computed();
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

        let context = Context::computed();
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

impl<T: Clone + PartialEq + 'static> From<Value<T>> for Computed<T> {
    fn from(val: Value<T>) -> Self {
        val.to_computed()
    }
}

impl<T: Clone + PartialEq + 'static> From<T> for Computed<T> {
    fn from(value: T) -> Self {
        Value::new(value).to_computed()
    }
}

impl<T: Clone + PartialEq + 'static> From<&T> for Computed<T> {
    fn from(value: &T) -> Self {
        Value::new(value.clone()).to_computed()
    }
}

impl From<&str> for Computed<String> {
    fn from(value: &str) -> Self {
        Value::new(value.to_string()).to_computed()
    }
}

#[test]
fn drop_computed() {
    let value = Value::new(3);

    let double = Computed::from({
        let value = value.clone();

        move |context| {
            let val = value.to_computed().get(context);
            val * 2
        }
    });

    let double_value = Rc::new(ValueMut::new(6));

    let drop_resource = double.subscribe({
        let double = double_value.clone();

        move |current| {
            double.set(current);
        }
    });

    assert_eq!(double_value.get(), 6);

    value.set(10);
    assert_eq!(double_value.get(), 20);

    drop(drop_resource);
}

#[test]
fn test_when_connect() {
    let connect_count = Rc::new(ValueMut::new(0));
    let disconnect_count = Rc::new(ValueMut::new(0));

    let value = Value::new(1);
    let comp = value.to_computed().when_connect({
        let connect_count = connect_count.clone();
        let disconnect_count = disconnect_count.clone();
        move || {
            connect_count.change(|v| *v += 1);
            DropResource::new({
                let disconnect_count = disconnect_count.clone();
                move || {
                    disconnect_count.change(|v| *v += 1);
                }
            })
        }
    });

    assert_eq!(connect_count.get(), 0);
    assert_eq!(disconnect_count.get(), 0);

    let drop_resource = comp.subscribe(|_| {});

    assert_eq!(connect_count.get(), 1);
    assert_eq!(disconnect_count.get(), 0);

    drop(drop_resource);

    assert_eq!(connect_count.get(), 1);
    assert_eq!(disconnect_count.get(), 1);
}

#[test]
fn test_when_connect_multiple() {
    let connect_count = Rc::new(ValueMut::new(0));
    let disconnect_count = Rc::new(ValueMut::new(0));

    let value = Value::new(1);
    let comp = value.to_computed().when_connect({
        let connect_count = connect_count.clone();
        let disconnect_count = disconnect_count.clone();
        move || {
            connect_count.change(|v| *v += 1);
            DropResource::new({
                let disconnect_count = disconnect_count.clone();
                move || {
                    disconnect_count.change(|v| *v += 1);
                }
            })
        }
    });

    let drop1 = comp.clone().subscribe(|_| {});
    assert_eq!(connect_count.get(), 1);

    let drop2 = comp.subscribe(|_| {});
    assert_eq!(connect_count.get(), 1);

    drop(drop1);
    assert_eq!(disconnect_count.get(), 0);

    drop(drop2);
    assert_eq!(disconnect_count.get(), 1);
}
