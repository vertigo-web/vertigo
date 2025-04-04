use std::hash::Hash;
use std::rc::Rc;

use crate::DomNode;
use crate::{
    computed::{Computed, Dependencies, GraphId, ToComputed},
    get_driver,
    struct_mut::ValueMut,
    DropResource,
};

use super::context::Context;

struct ValueInner<T> {
    id: GraphId,
    value: ValueMut<T>,
    deps: &'static Dependencies,
}

impl<T> Drop for ValueInner<T> {
    fn drop(&mut self) {
        self.deps
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
        let deps = get_driver().inner.dependencies;
        Value {
            inner: Rc::new(ValueInner {
                id: GraphId::new_value(),
                value: ValueMut::new(value),
                deps,
            }),
        }
    }

    /// Create a value that is connected to a generator, where `value` parameter is a starting value,
    /// and `create` function takes care of updating it.
    ///
    /// See [game of life](../src/vertigo_demo/app/game_of_life/mod.rs.html#54) example.
    pub fn with_connect<F>(value: T, create: F) -> Computed<T>
    where
        F: Fn(&Value<T>) -> DropResource + 'static,
    {
        let deps = get_driver().inner.dependencies;
        let id = GraphId::new_value();

        let value = Value {
            inner: Rc::new(ValueInner {
                id,
                value: ValueMut::new(value),
                deps,
            }),
        };

        let computed = value.to_computed();

        deps.graph
            .external_connections
            .register_connect(id, Rc::new(move || create(&value)));

        computed
    }

    pub fn set(&self, value: T) {
        self.inner.deps.transaction(|_| {
            self.inner.value.set(value);
            self.inner.deps.report_set(self.inner.id);
        });
    }

    pub fn get(&self, context: &Context) -> T {
        context.add_parent(self.inner.id);
        self.inner.value.get()
    }

    pub fn change(&self, change_fn: impl FnOnce(&mut T)) {
        self.inner.deps.transaction(|ctx| {
            let mut value = self.get(ctx);
            change_fn(&mut value);
            self.set(value);
        });
    }

    pub fn map<K: Clone + 'static, F: 'static + Fn(T) -> K>(&self, fun: F) -> Computed<K> {
        Computed::from({
            let computed = self.clone();
            move |context| fun(computed.get(context))
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id
    }

    pub fn deps(&self) -> &'static Dependencies {
        self.inner.deps
    }

    pub fn to_computed(&self) -> Computed<T> {
        let self_clone = self.clone();

        Computed::from(move |context| self_clone.get(context))
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
    pub fn set_value_and_compare(&self, value: T) {
        self.inner.deps.transaction(|_| {
            let need_refresh = self.inner.value.set_and_check(value);
            if need_refresh {
                self.inner.deps.report_set(self.inner.id);
            }
        });
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
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
