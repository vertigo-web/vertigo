use std::rc::Rc;
use std::cmp::PartialEq;
use std::any::Any;

use crate::computed::{
    Dependencies,
    Computed,
    GraphId,
};
use crate::utils::BoxRefCell;

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

struct ValueInner<T: PartialEq + 'static> {
    id: GraphId,
    value: Rc<T>,
    deps: Dependencies,
}

impl<T: PartialEq + 'static> ValueInner<T> {
    fn set_value(&mut self, value: T) {
        if *(self.value) == value {
            return;
        }

        self.value = Rc::new(value);
        self.deps.trigger_change(self.id);
    }

    fn get_value(&self) -> Rc<T> {
        self.deps.report_parent_in_stack(self.id);
        self.value.clone()
    }
}

impl<T: PartialEq + 'static> Drop for ValueInner<T> {
    fn drop(&mut self) {
        self.deps.external_connections.unregister_connect(self.id);
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
pub struct Value<T: PartialEq + 'static> {
    inner: Rc<BoxRefCell<ValueInner<T>>>,
    pub deps: Dependencies,
}

impl<T: PartialEq + 'static> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            inner: self.inner.clone(),
            deps: self.deps.clone(),
        }
    }
}

impl<T: PartialEq + 'static> Value<T> {
    pub fn new(deps: Dependencies, value: impl ToRc<T>) -> Value<T> {
        Value {
            inner: Rc::new(BoxRefCell::new(
                ValueInner {
                    id: GraphId::default(),
                    value: value.to_rc(),
                    deps: deps.clone(),
                },
                "value inner"
            )),
            deps,
        }
    }

    pub fn new_selfcomputed_value<F: Fn(&Value<T>) -> Box<dyn Any> + 'static>(deps: Dependencies, value: T, create: F) -> Computed<T> {
        let id = GraphId::default();

        let value = Value {
            inner: Rc::new(BoxRefCell::new(
                ValueInner {
                    id,
                    value: Rc::new(value),
                    deps: deps.clone(),
                },
                "value inner connect"
            )),
            deps: deps.clone(),
        };

        let computed = value.to_computed();

        deps.external_connections.register_connect(id, Box::new(move || {
            create(&value)
        }));

        computed
    }

    pub fn set_value(&self, value: T) {
        self.deps.clone().transaction(|| {
            self.inner.change(value, move |state, value| {
                state.set_value(value);
            })
        })
    }

    pub fn get_value(&self) -> Rc<T> {
        self.inner.get(|state| state.get_value())
    }

    pub fn to_computed(&self) -> Computed<T> {
        let inner_clone = self.inner.clone();

        let deps = self.deps.clone();

        Computed::new(deps, move || {
            inner_clone.get(|state| state.get_value())
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.get(|state| state.id)
    }
}

impl<T: PartialEq + 'static> PartialEq for Value<T> {
    fn eq(&self, other: &Value<T>) -> bool {
        self.id() == other.id()
    }
}
