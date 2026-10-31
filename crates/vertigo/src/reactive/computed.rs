use std::{cell::RefCell, ops::Add, rc::Rc};

use super::{
    Context, DropResource, Graph, GraphId, ToComputed, Value,
    graph::{CallbackGuard, ErasedNode, GraphInner, Node, NodeId},
};

/// A read-only reactive cell, recomputed from other nodes.
pub struct Computed<T: Clone + PartialEq + 'static> {
    inner: Rc<ComputedInner<T>>,
}

struct ComputedInner<T: Clone + PartialEq + 'static> {
    node: Node,
    compute: Box<dyn Fn(&Context) -> T>,
    value: RefCell<Option<T>>,
}

struct SubscribeInner {
    node: Node,
    refresh: Box<dyn Fn(&Context)>,
}

impl<T: Clone + PartialEq + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + PartialEq + 'static> PartialEq for Computed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.node.id == other.inner.node.id
    }
}

impl<T: Clone + PartialEq + 'static> ErasedNode for ComputedInner<T> {
    fn refresh(&self) -> bool {
        self.recompute()
    }
}

impl ErasedNode for SubscribeInner {
    fn refresh(&self) -> bool {
        let tracking = Tracking::start(&self.node.graph);
        (self.refresh)(&tracking.context);
        tracking.record(self.node.id);
        false
    }
}

impl<T: Clone + PartialEq + 'static> ComputedInner<T> {
    fn recompute(&self) -> bool {
        let tracking = Tracking::start(&self.node.graph);
        let new_value = (self.compute)(&tracking.context);
        tracking.record(self.node.id);

        let mut slot = self.value.borrow_mut();
        match slot.as_ref() {
            Some(old) if old == &new_value => false,
            _ => {
                *slot = Some(new_value);
                true
            }
        }
    }

    fn ensure(&self) -> T {
        if self.value.borrow().is_none() {
            self.recompute();
        }
        match self.value.borrow().clone() {
            Some(value) => value,
            None => no_value(),
        }
    }
}

/// A compute running: what it reads is recorded as the parents of its node, and writes are
/// blocked until it's dropped. The same for every type, so starting it and recording are out
/// of line, not in each copy of [`ComputedInner::recompute`].
struct Tracking<'a> {
    graph: &'a GraphInner,
    _guard: CallbackGuard<'a>,
    context: Context,
}

impl<'a> Tracking<'a> {
    #[inline(never)]
    fn start(graph: &'a GraphInner) -> Self {
        Tracking {
            graph,
            _guard: graph.enter_callback(),
            context: Context::tracking(),
        }
    }

    #[inline(never)]
    fn record(&self, id: NodeId) {
        self.graph.set_parents(id, self.context.take_parents());
    }
}

#[cold]
#[inline(never)]
fn no_value() -> ! {
    panic!("vertigo: computed has no value after refresh")
}

impl<T: Clone + PartialEq + 'static> Computed<T> {
    /// A computed on `graph`, with the closure already in the box `ComputedInner` stores it
    /// in anyway.
    ///
    /// Generic over `T` but not over the closure type: every `Computed::from`, `map` and
    /// `to_computed` in an application funnels through here, each boxing its closure itself,
    /// so that none of them gets a copy of the registration below, or a layer of its own
    /// between it and here.
    pub(crate) fn create(graph: Rc<GraphInner>, compute: Box<dyn Fn(&Context) -> T>) -> Self {
        let inner = Rc::new(ComputedInner {
            node: Node::new(graph),
            compute,
            value: RefCell::new(None),
        });
        inner.node.graph.register(inner.node.id, inner.clone());
        Computed { inner }
    }

    pub fn from(compute: impl Fn(&Context) -> T + 'static) -> Self {
        Self::create(super::default_graph().inner, Box::new(compute))
    }

    pub fn get(&self, ctx: &Context) -> T {
        self.inner.node.read(ctx, self.inner.clone());
        self.inner.ensure()
    }

    pub fn id(&self) -> GraphId {
        GraphId::from_node(self.inner.node.id)
    }

    /// Runs `create` after the wave (and after `on_after_transaction`) in which this
    /// value starts being observed. The returned [`DropResource`] is dropped after the
    /// wave in which it stops being observed; that drop must only tear down the external
    /// handler, it must not write. `create` may write `Value`s. A `set` from compute or
    /// subscribe is ignored.
    pub fn when_connect<F: Fn() -> DropResource + 'static>(&self, create: F) -> Computed<T> {
        let new_computed = Computed::create(self.inner.node.graph.clone(), {
            let parent = self.clone();
            Box::new(move |context| parent.get(context))
        });
        new_computed
            .inner
            .node
            .graph
            .register_connect(new_computed.inner.node.id, Rc::new(create));
        new_computed
    }

    /// Subscribe; the callback runs only when the computed value *changes*.
    pub fn subscribe<R: 'static, F: Fn(T) -> R + 'static>(self, callback: F) -> DropResource {
        let graph = self.inner.node.graph.clone();
        let parent = self;

        // Everything below this line is the same code whatever `T` and whatever the callback
        // is, so it lives in a function that is generic over neither - see `register_sink`.
        register_sink(
            graph,
            Box::new(move |ctx| {
                let _ = callback(parent.get(ctx));
            }),
        )
    }

    pub fn map<K: Clone + PartialEq + 'static, F: 'static + Fn(T) -> K>(
        &self,
        fun: F,
    ) -> Computed<K> {
        Computed::create(self.inner.node.graph.clone(), {
            let myself = self.clone();
            Box::new(move |context| fun(myself.get(context)))
        })
    }
}

impl<T: Clone + PartialEq + 'static> Computed<T> {
    /// The graph this computed lives on.
    pub(crate) fn graph(&self) -> Graph {
        Graph {
            inner: self.inner.node.graph.clone(),
        }
    }
}

impl Graph {
    /// [`Computed::subscribe`] for a callback that reads what it needs from the graph itself,
    /// so that the caller doesn't have to be generic over a value's type for the subscription.
    pub(crate) fn subscribe(&self, refresh: Box<dyn Fn(&Context)>) -> DropResource {
        register_sink(self.inner.clone(), refresh)
    }
}

/// Registers a subscription sink on the graph and runs it once.
///
/// Split out of [`Computed::subscribe`] and generic over nothing at all.
fn register_sink(graph: Rc<GraphInner>, refresh: Box<dyn Fn(&Context)>) -> DropResource {
    let inner = Rc::new(SubscribeInner {
        node: Node::new(graph.clone()),
        refresh,
    });
    graph.register(inner.node.id, inner.clone());

    // The first run has to be a transaction: registering the parents it read is what
    // makes them watched, and `when_connect` closures only run when a transaction
    // closes. Refreshing bare would leave them queued until some later, unrelated
    // transaction happened to flush them. Subscribing from inside a wave is fine -
    // that transaction closes without flushing, and the running wave does it at the end.
    Graph {
        inner: graph.clone(),
    }
    .transaction(|_| {
        inner.refresh();
    });

    DropResource::from_struct(inner)
}

impl<T: Clone + PartialEq + 'static> ToComputed<T> for Computed<T> {
    fn to_computed(&self) -> Computed<T> {
        self.clone()
    }
}

impl<T: Clone + PartialEq + 'static> ToComputed<T> for &Computed<T> {
    fn to_computed(&self) -> Computed<T> {
        (*self).clone()
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

impl<T> Add for Computed<T>
where
    T: Clone + PartialEq + Add<Output = T> + 'static,
{
    type Output = Computed<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Computed::from({
            let left = self;
            let right = rhs;
            move |ctx| left.get(ctx) + right.get(ctx)
        })
    }
}

impl<T> Add<T> for Computed<T>
where
    T: Clone + PartialEq + Add<Output = T> + 'static,
{
    type Output = Computed<T>;

    fn add(self, rhs: T) -> Self::Output {
        self.map(move |left| left + rhs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Graph, *};
    use std::cell::Cell;

    #[test]
    fn subscribe_does_not_notify_dependents() {
        let g = Graph::new();
        let logs = g.logger().listen();
        let a = g.value(0);
        let node = Node::new(g.inner.clone());
        let id = node.id;
        let sink = Rc::new(SubscribeInner {
            node,
            refresh: Box::new({
                let a = a.clone();
                move |ctx| {
                    let _ = a.get(ctx);
                }
            }),
        });
        g.inner.register(id, sink.clone());
        sink.refresh();

        let runs = Rc::new(Cell::new(0));
        let child = g.computed({
            let sink = sink.clone();
            let runs = runs.clone();
            move |ctx| {
                runs.set(runs.get() + 1);
                ctx.track(id, sink.clone());
                0
            }
        });
        g.transaction(|ctx| {
            let _ = child.get(ctx);
        });
        runs.set(0);
        a.set(1);
        assert_eq!(runs.get(), 0);
        logs.assert_eq(&[]);
    }
}
