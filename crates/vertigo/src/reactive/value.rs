use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::{
    Computed, Context, DropResource, Graph, GraphId, ToComputed,
    graph::{ErasedNode, GraphInner, NodeId},
};

/// The listeners from [`Value::add_event`]. Each one reads the new value from the cell itself,
/// so that calling them is the same code whatever `T` is.
type ValueEvents = Vec<(u64, Rc<dyn Fn()>)>;

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
    events: RefCell<ValueEvents>,
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
            events: RefCell::new(Vec::new()),
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
    /// `create` runs after the wave in which this value starts being observed (and after
    /// `on_after_transaction`), so it may call [`Value::set`](Self::set). The returned
    /// [`DropResource`] must only tear down the external handler; a `set` from drop is
    /// ignored. A `set` from compute or subscribe is ignored.
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
            // Listeners clone it from the cell, each its own copy - with none, nothing copies
            // whatever is stored.
            *self.inner.value.borrow_mut() = value;
            notify(&self.inner.events);
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

    /// Calls `callback` after every [`Value::set`] that changes the value, until the returned
    /// [`DropResource`] is dropped. It gets a copy of what the value holds then - when an
    /// earlier listener has set it once more, that newer value.
    pub fn add_event(&self, callback: impl Fn(T) + 'static) -> DropResource {
        let id = {
            let mut next = self.inner.next_event.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };
        // Weak: the list is part of the value, which mustn't keep itself alive
        let value: Weak<ValueInner<T>> = Rc::downgrade(&self.inner);
        let event = move || {
            if let Some(value) = value.upgrade() {
                let current = value.value.borrow().clone();
                callback(current);
            }
        };
        self.inner.events.borrow_mut().push((id, Rc::new(event)));
        let inner = self.inner.clone();
        DropResource::new(move || {
            inner
                .events
                .borrow_mut()
                .retain(|(event_id, _)| *event_id != id);
        })
    }
}

/// Calls the listeners of a value that has just changed. They run from a copy of the list, so
/// that each may add or drop listeners.
fn notify(events: &RefCell<ValueEvents>) {
    if events.borrow().is_empty() {
        return;
    }
    let events: Vec<_> = events
        .borrow()
        .iter()
        .map(|(_, event)| event.clone())
        .collect();
    for event in events {
        event();
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
