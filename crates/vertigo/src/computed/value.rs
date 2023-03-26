use std::{
    cmp::PartialEq,
    rc::Rc,
};
use std::hash::Hash;
use crate::DomNode;
use crate::dom::dom_comment::DomComment;
use crate::{
    computed::{Computed, Dependencies, GraphId}, struct_mut::ValueMut, DropResource,
    get_driver,
};

use super::context::Context;

struct ValueInner<T> {
    id: GraphId,
    value: ValueMut<T>,
    deps: &'static Dependencies,
}

impl<T> Drop for ValueInner<T> {
    fn drop(&mut self) {
        self.deps.graph.external_connections.unregister_connect(self.id);
    }
}

/// A reactive value. Basic building block of app state.
///
/// Can be read or written.
///
/// ```rust
/// use vertigo::{Computed, Value, transaction};
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
pub struct Value<T> {
    inner: Rc<ValueInner<T>>,
}

impl<T: Clone + Default + 'static> Default for Value<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            inner: self.inner.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id.eq(&other.inner.id)
    }
}

impl<T: Clone + 'static> Value<T> {
    pub fn new(value: T) -> Value<T> {
        let deps = get_driver().inner.dependencies;
        Value {
            inner: Rc::new(
                ValueInner {
                    id: GraphId::new_value(),
                    value: ValueMut::new(value),
                    deps,
                }
            )
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
            inner: Rc::new(
                ValueInner {
                    id,
                    value: ValueMut::new(value),
                    deps,
                },
            )
        };

        let computed = value.to_computed();

        deps.graph.external_connections.register_connect(id, Rc::new(move || {
            create(&value)
        }));

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
            move |context| {
                fun(computed.get(context))
            }
        })
    }

    pub fn to_computed(&self) -> Computed<T> {
        let self_clone = self.clone();

        Computed::from(move |context| {
            self_clone.get(context)
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id
    }

    pub fn deps(&self) -> &'static Dependencies {
        self.inner.deps
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
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        self.to_computed().render_value(render)
    }

    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomComment {
        self.to_computed().render_value_option(render)
    }
}

impl<
    T: PartialEq + Clone + 'static,
    L: IntoIterator<Item=T> + Clone + PartialEq + 'static
> Value<L> {
    pub fn render_list<
        K: Eq + Hash,
    >(
        &self,
        get_key: impl Fn(&T) -> K + 'static,
        render: impl Fn(&T) -> DomNode + 'static,
    ) -> DomComment {
        self.to_computed().render_list(get_key, render)
    }
}
