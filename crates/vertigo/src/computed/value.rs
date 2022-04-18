use std::{
    cmp::PartialEq,
    rc::Rc,
};

use crate::{
    computed::{Computed, Dependencies, GraphId}, struct_mut::ValueMut, DropResource,
};

pub trait ToRc<T> {
    fn to_rc(self) -> Rc<T>;
}

impl<T> ToRc<T> for Rc<T> {
    fn to_rc(self) -> Rc<T> {
        self
    }
}

impl<T> ToRc<T> for T {
    fn to_rc(self) -> Rc<T> {
        Rc::new(self)
    }
}

struct ValueInner<T> {
    id: GraphId,
    value: ValueMut<Rc<T>>,
    deps: Dependencies,
}

impl<T> Drop for ValueInner<T> {
    fn drop(&mut self) {
        self.deps.external_connections_unregister_connect(self.id);
    }
}

/// A reactive value. Basic building block of app state.
///
/// Can be read or written.
///
/// ```rust
/// use vertigo::{Computed, Dependencies};
///
/// let deps = Dependencies::default();
///
/// let value = deps.new_value(5);
///
/// assert_eq!(*value.get_value(), 5);
///
/// value.set_value(10);
///
/// assert_eq!(*value.get_value(), 10);
/// ```
///
pub struct Value<T> {
    inner: Rc<ValueInner<T>>,
}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Value<T> {
    pub fn new(deps: Dependencies, value: impl ToRc<T>) -> Value<T> {
        Value {
            inner: Rc::new(
                ValueInner {
                    id: GraphId::default(),
                    value: ValueMut::new(value.to_rc()),
                    deps,
                }
            )
        }
    }

    pub fn new_selfcomputed_value<F>(deps: Dependencies, value: T, create: F) -> Computed<T>
    where
        F: Fn(&Value<T>) -> DropResource + 'static,
    {
        let id = GraphId::default();

        let value = Value {
            inner: Rc::new(
                ValueInner {
                    id,
                    value: ValueMut::new(Rc::new(value)),
                    deps: deps.clone(),
                },
            )
        };

        let computed = value.to_computed();

        deps.external_connections_register_connect(id, Rc::new(move || {
            create(&value)
        }));

        computed
    }

    pub fn set_value(&self, value: T) {
        self.inner.deps.clone().transaction(|| {
            self.inner.value.set(Rc::new(value));
            self.inner.deps.trigger_change(self.inner.id);
        });
    }

    pub fn get_value(&self) -> Rc<T> {
        self.inner.deps.report_parent_in_stack(self.inner.id);
        self.inner.value.get()
    }

    pub fn to_computed(&self) -> Computed<T> {
        let self_clone = self.clone();

        Computed::new(self.deps(), move || {
            self_clone.get_value()
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id
    }

    pub fn deps(&self) -> Dependencies {
        self.inner.deps.clone()
    }
}

impl<T: PartialEq + 'static> Value<T> {
    pub fn set_value_and_compare(&self, value: T) {
        self.inner.deps.clone().transaction(|| {
            let need_update = self.inner.value.set_and_check(Rc::new(value));

            if need_update {
                self.inner.deps.trigger_change(self.inner.id);
            }
        });
    }
}
