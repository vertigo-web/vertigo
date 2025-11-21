use std::hash::Hash;
use std::rc::Rc;

use crate::DomNode;
use crate::computed::dependencies::get_dependencies;
use crate::{
    computed::{Computed, GraphId, ToComputed},
    struct_mut::ValueMut,
    DropResource,
};

use super::context::Context;

struct ValueInner<T> {
    id: GraphId,
    value: ValueMut<T>,
}

impl<T> Drop for ValueInner<T> {
    fn drop(&mut self) {
        get_dependencies()
            .graph
            .external_connections
            .unregister_connect(self.id);
    }
}

/// A reactive value. Basic building block of app state.
///
/// Can be read or written.
///
/// ```rust
/// use vertigo::{Value, transaction};
///
/// let value = Value::new(5);
///
/// transaction(|context| {
///     assert_eq!(value.get(context), 5);
/// });
///
/// value.set(10);
///
/// transaction(|context| {
///     assert_eq!(value.get(context), 10);
/// });
/// ```
///
#[derive(Clone)]
pub struct Value<T> {
    inner: Rc<ValueInner<T>>,
}

impl<T: Clone + Default + 'static> Default for Value<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: PartialEq> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id.eq(&other.inner.id)
    }
}

impl<T: Clone + 'static> Value<T> {
    pub fn new(value: T) -> Self {
        Value {
            inner: Rc::new(ValueInner {
                id: GraphId::new_value(),
                value: ValueMut::new(value),
            }),
        }
    }

    /// Create a value that is connected to a generator, where `value` parameter is a starting value,
    /// and `create` function takes care of updating it.
    ///
    /// See [Router implementation](../src/vertigo/router.rs.html#97) for example example.
    pub fn with_connect<F>(value: T, create: F) -> Computed<T>
    where
        F: Fn(&Value<T>) -> DropResource + 'static,
    {
        let id = GraphId::new_value();

        let value = Value {
            inner: Rc::new(ValueInner {
                id,
                value: ValueMut::new(value),
            }),
        };

        let computed = value.to_computed();

        get_dependencies().graph
            .external_connections
            .register_connect(id, Rc::new(move || create(&value)));

        computed
    }

    /// Allows to set a new value if `T` doesn't implement [PartialEq].
    ///
    /// This will always trigger a graph change even if the value stays the same.
    pub fn set_force(&self, value: T) {
        get_dependencies().transaction(|_| {
            self.inner.value.set(value);
            get_dependencies().report_set(self.inner.id);
        });
    }

    /// Get the value.
    ///
    /// Use this in callbacks. You can get [Context] object using [transaction](crate::transaction) function.
    /// During rendering you should directly embed the `Value` in [dom!](crate::dom) or use [.render_value()](Value::render_value) method.
    ///
    /// Returned `T` is cloned - it's not reactive.
    pub fn get(&self, context: &Context) -> T {
        context.add_parent(self.inner.id);
        self.inner.value.get()
    }

    /// Reactively convert `Value` into [Computed] with provided transformation function applied.
    pub fn map<K: Clone + 'static, F: 'static + Fn(T) -> K>(&self, fun: F) -> Computed<K> {
        Computed::from({
            let myself = self.clone();
            move |context| fun(myself.get(context))
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id
    }

    /// Reactively convert the `Value`` into [Computed] without any mapping.
    pub fn to_computed(&self) -> Computed<T> {
        let myself = self.clone();

        Computed::from(move |context| myself.get(context))
    }
}

impl<T: Clone + 'static> ToComputed<T> for Value<T> {
    fn to_computed(&self) -> Computed<T> {
        self.to_computed()
    }
}

impl<T: Clone + 'static> ToComputed<T> for &Value<T> {
    fn to_computed(&self) -> Computed<T> {
        (*self).to_computed()
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
    pub fn change(&self, change_fn: impl FnOnce(&mut T)) {
        get_dependencies().transaction(|ctx| {
            let mut value = self.get(ctx);
            change_fn(&mut value);
            self.set(value);
        });
    }

    pub fn set(&self, value: T) {
        get_dependencies().transaction(|_| {
            let need_refresh = self.inner.value.set_if_changed(value);
            if need_refresh {
                get_dependencies().report_set(self.inner.id);
            }
        });
    }

    /// Render value (reactively transforms `T` into `DomNode`)
    ///
    /// See [computed_tuple](macro.computed_tuple.html) if you want to render multiple values in a handy way.
    ///
    /// ```rust
    /// use vertigo::{dom, Value};
    ///
    /// let my_value = Value::new(5);
    ///
    /// let element = my_value.render_value(|bare_value| dom! { <div>{bare_value}</div> });
    ///
    /// dom! {
    ///     <div>
    ///         {element}
    ///     </div>
    /// };
    /// ```
    ///
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        self.to_computed().render_value(render)
    }

    /// Render optional value (reactively transforms `Option<T>` into `Option<DomNode>`)
    ///
    /// See [computed_tuple](macro.computed_tuple.html) if you want to render multiple values in a handy way.
    ///
    /// ```rust
    /// use vertigo::{dom, Value};
    ///
    /// let value1 = Value::new(Some(5));
    /// let value2 = Value::new(None::<i32>);
    ///
    /// let element1 = value1.render_value_option(|bare_value|
    ///     bare_value.map(|value| dom! { <div>{value}</div> })
    /// );
    /// let element2 = value2.render_value_option(|bare_value|
    ///     match bare_value {
    ///         Some(value) => Some(dom! { <div>{value}</div> }),
    ///         None => Some(dom! { <div>"default"</div> }),
    ///     }
    /// );
    ///
    /// dom! {
    ///     <div>
    ///         {element1}
    ///         {element2}
    ///     </div>
    /// };
    /// ```
    ///
    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        self.to_computed().render_value_option(render)
    }
}

impl<T: PartialEq + Clone + 'static, L: IntoIterator<Item = T> + Clone + PartialEq + 'static>
    Value<L>
{
    /// Render iterable value (reactively transforms `Iterator<T>` into Node with list of rendered elements )
    ///
    /// ```rust
    /// use vertigo::{dom, Value};
    ///
    /// let my_list = Value::new(vec![
    ///     (1, "one"),
    ///     (2, "two"),
    ///     (3, "three"),
    /// ]);
    ///
    /// let elements = my_list.render_list(
    ///     |el| el.0,
    ///     |el| dom! { <div>{el.1}</div> }
    /// );
    ///
    /// dom! {
    ///     <div>
    ///         {elements}
    ///     </div>
    /// };
    /// ```
    ///
    pub fn render_list<K: Eq + Hash>(
        &self,
        get_key: impl Fn(&T) -> K + 'static,
        render: impl Fn(&T) -> DomNode + 'static,
    ) -> DomNode {
        self.to_computed().render_list(get_key, render)
    }
}
