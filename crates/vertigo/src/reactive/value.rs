use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::{
    Computed, Context, DropResource, Graph, GraphId, ToComputed,
    graph::{ErasedNode, GraphInner, NodeId},
};

type ValueEvents<T> = BTreeMap<u64, Rc<dyn Fn(T)>>;

/// A writable reactive cell.
///
/// ```
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
pub struct Value<T: Clone + PartialEq + 'static> {
    inner: Rc<ValueInner<T>>,
}

struct ValueInner<T: Clone + PartialEq + 'static> {
    graph: Rc<GraphInner>,
    id: NodeId,
    value: RefCell<T>,
    next_event: RefCell<u64>,
    events: RefCell<ValueEvents<T>>,
}

impl<T: Clone + PartialEq + 'static> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + PartialEq + 'static> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl<T: Clone + PartialEq + Default + 'static> Default for Value<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone + PartialEq + 'static> ErasedNode for ValueInner<T> {
    fn refresh(&self) -> bool {
        true
    }
}

impl<T: Clone + PartialEq + 'static> Drop for ValueInner<T> {
    fn drop(&mut self) {
        self.graph.unregister(self.id);
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
    pub(crate) fn create(graph: Rc<GraphInner>, value: T) -> Self {
        let id = graph.alloc_id();
        let inner = Rc::new(ValueInner {
            graph: graph.clone(),
            id,
            value: RefCell::new(value),
            next_event: RefCell::new(1),
            events: RefCell::new(BTreeMap::new()),
        });
        graph.register(id, inner.clone());
        Value { inner }
    }

    /// Create a value on the default graph.
    pub fn new(value: T) -> Self {
        super::default_graph().value(value)
    }

    /// Create a value that is connected to a generator. `value` is the starting
    /// value; `create` is responsible for keeping it up to date.
    ///
    /// `create` runs after the wave in which this value starts being observed, so it
    /// may call [`Value::set`](Self::set). A `set` from compute or subscribe is ignored.
    pub fn with_connect<F>(value: T, create: F) -> Computed<T>
    where
        F: Fn(&Value<T>) -> DropResource + 'static,
    {
        let value = Value::new(value);
        let value_clone = value.clone();
        value
            .to_computed()
            .when_connect(move || create(&value_clone))
    }

    pub fn get(&self, ctx: &Context) -> T {
        ctx.track(self.inner.id, self.inner.clone());
        self.inner.graph.ensure_fresh(self.inner.id);
        self.inner.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        // Before the equality check below, deliberately: writing from a place that must
        // not write is reported even when the new value happens to equal the old one.
        // Otherwise the same broken code would report itself or not depending on the data.
        if !self.inner.graph.check_write_allowed() {
            return;
        }
        let graph = Graph {
            inner: self.inner.graph.clone(),
        };
        graph.transaction(|_| {
            if *self.inner.value.borrow() == value {
                return;
            }
            // The clone only exists to hand the new value to the listeners, so skip it
            // when there are none — otherwise every write deep-copies whatever is stored.
            if self.inner.events.borrow().is_empty() {
                *self.inner.value.borrow_mut() = value;
            } else {
                *self.inner.value.borrow_mut() = value.clone();
                let events: Vec<_> = self.inner.events.borrow().values().cloned().collect();
                for event in events {
                    event(value.clone());
                }
            }
            self.inner.graph.enqueue(self.inner.id);
        });
    }

    pub fn change(&self, change_fn: impl FnOnce(&mut T)) {
        let graph = Graph {
            inner: self.inner.graph.clone(),
        };
        graph.transaction(|ctx| {
            let mut value = self.get(ctx);
            change_fn(&mut value);
            self.set(value);
        });
    }

    pub fn map<K: Clone + PartialEq + 'static, F: 'static + Fn(T) -> K>(
        &self,
        fun: F,
    ) -> Computed<K> {
        Computed::create(self.inner.graph.clone(), {
            let myself = self.clone();
            move |context| fun(myself.get(context))
        })
    }

    pub fn to_computed(&self) -> Computed<T> {
        let myself = self.clone();
        Computed::create(self.inner.graph.clone(), move |context| myself.get(context))
    }

    pub fn id(&self) -> GraphId {
        GraphId::from_node(self.inner.id)
    }

    pub fn add_event(&self, callback: impl Fn(T) + 'static) -> DropResource {
        let id = {
            let mut next = self.inner.next_event.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };
        self.inner.events.borrow_mut().insert(id, Rc::new(callback));
        let inner = self.inner.clone();
        DropResource::new(move || {
            inner.events.borrow_mut().remove(&id);
        })
    }
}

impl<T: Clone + PartialEq + 'static> ToComputed<T> for Value<T> {
    fn to_computed(&self) -> Computed<T> {
        self.to_computed()
    }
}

impl<T: Clone + PartialEq + 'static> ToComputed<T> for &Value<T> {
    fn to_computed(&self) -> Computed<T> {
        (*self).to_computed()
    }
}
