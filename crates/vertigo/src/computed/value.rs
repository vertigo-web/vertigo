use std::{
    cmp::PartialEq,
    rc::Rc,
};
use std::hash::Hash;
use crate::{DomElement, get_driver};
use crate::dom::dom_comment_create::DomCommentCreate;
use crate::dom::dom_node::DomNodeFragment;
use crate::dom_list::ListRendered;
use crate::{
    computed::{Computed, Dependencies, GraphId}, struct_mut::ValueMut, DropResource,
};

use super::context::Context;

struct ValueInner<T> {
    id: GraphId,
    value: ValueMut<T>,
    deps: Dependencies,
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
        let deps = get_driver().inner.dependencies.clone();
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
        let deps = get_driver().inner.dependencies.clone();
        let id = GraphId::new_value();

        let value = Value {
            inner: Rc::new(
                ValueInner {
                    id,
                    value: ValueMut::new(value),
                    deps: deps.clone(),
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
            let set_value = {
                let id = self.inner.id;
                let inner = self.inner.clone();
                move || {
                    inner.value.set(value);
                    Some(id)
                }
            };

            self.inner.deps.transaction_state.add_edge_to_refresh(set_value);
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

    pub fn deps(&self) -> Dependencies {
        self.inner.deps.clone()
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
    pub fn set_value_and_compare(&self, value: T) {
        self.inner.deps.transaction(|_| {

            let set_value = {
                let id = self.inner.id;
                let inner = self.inner.clone();
                move || {

                    let need_update = inner.value.set_and_check(value);
                    if need_update {
                        Some(id)
                    } else {
                        None
                    }
                }
            };

            self.inner.deps.transaction_state.add_edge_to_refresh(set_value);
        });
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
    pub fn render_value<R: Into<DomNodeFragment>>(&self, render: impl Fn(T) -> R + 'static) -> DomCommentCreate {
        self.to_computed().render_value(render)
    }

    pub fn render_value_option<R: Into<DomNodeFragment>>(&self, render: impl Fn(T) -> Option<R> + 'static) -> DomCommentCreate {
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
        render: impl Fn(&T) -> DomElement + 'static,
    ) -> ListRendered<T> {
        self.to_computed().render_list(get_key, render)
    }
}
