use std::{cell::RefCell, ops::Add, rc::Rc};

use super::{
    Context, DropResource, GraphId, ToComputed, Value,
    graph::{ErasedNode, GraphInner, NodeId},
};

/// A read-only reactive cell, recomputed from other nodes.
pub struct Computed<T: Clone + PartialEq + 'static> {
    inner: Rc<ComputedInner<T>>,
}

struct ComputedInner<T: Clone + PartialEq + 'static> {
    graph: Rc<GraphInner>,
    id: NodeId,
    compute: Box<dyn Fn(&Context) -> T>,
    value: RefCell<Option<T>>,
}

struct SubscribeInner {
    graph: Rc<GraphInner>,
    id: NodeId,
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
        self.inner.id == other.inner.id
    }
}

impl<T: Clone + PartialEq + 'static> ErasedNode for ComputedInner<T> {
    fn refresh(&self) -> bool {
        self.recompute()
    }
}

impl ErasedNode for SubscribeInner {
    fn refresh(&self) -> bool {
        let ctx = Context::tracking();
        (self.refresh)(&ctx);
        self.graph.set_parents(self.id, ctx.take_parents());
        false
    }
}

impl<T: Clone + PartialEq + 'static> Drop for ComputedInner<T> {
    fn drop(&mut self) {
        self.graph.unregister(self.id);
    }
}

impl Drop for SubscribeInner {
    fn drop(&mut self) {
        self.graph.unregister(self.id);
    }
}

impl<T: Clone + PartialEq + 'static> ComputedInner<T> {
    fn recompute(&self) -> bool {
        let ctx = Context::tracking();
        let new_value = (self.compute)(&ctx);
        self.graph.set_parents(self.id, ctx.take_parents());

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
            None => panic!("vertigo: computed has no value after refresh"),
        }
    }
}

impl<T: Clone + PartialEq + 'static> Computed<T> {
    pub(crate) fn create(graph: Rc<GraphInner>, compute: impl Fn(&Context) -> T + 'static) -> Self {
        let id = graph.alloc_id();
        let inner = Rc::new(ComputedInner {
            graph: graph.clone(),
            id,
            compute: Box::new(compute),
            value: RefCell::new(None),
        });
        graph.register(id, inner.clone());
        Computed { inner }
    }

    pub fn from(compute: impl Fn(&Context) -> T + 'static) -> Self {
        super::default_graph().computed(compute)
    }

    pub fn get(&self, ctx: &Context) -> T {
        ctx.track(self.inner.id, self.inner.clone());
        self.inner.ensure()
    }

    pub fn id(&self) -> GraphId {
        GraphId::from_node(self.inner.id)
    }

    /// Runs `create` when this value starts being observed; the returned
    /// [`DropResource`] is dropped when it stops being observed.
    pub fn when_connect<F: Fn() -> DropResource + 'static>(&self, create: F) -> Computed<T> {
        let new_computed = Computed::create(self.inner.graph.clone(), {
            let parent = self.clone();
            move |context| parent.get(context)
        });
        new_computed
            .inner
            .graph
            .register_connect(new_computed.inner.id, Rc::new(create));
        new_computed
    }

    /// Subscribe; the callback runs only when the computed value *changes*.
    pub fn subscribe<R: 'static, F: Fn(T) -> R + 'static>(self, callback: F) -> DropResource {
        let graph = self.inner.graph.clone();
        let parent = self.clone();
        let id = graph.alloc_id();
        let inner = Rc::new(SubscribeInner {
            graph: graph.clone(),
            id,
            refresh: Box::new(move |ctx| {
                let _ = callback(parent.get(ctx));
            }),
        });
        graph.register(id, inner.clone());
        inner.refresh();
        DropResource::from_struct(inner)
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
        let a = g.value(0);
        let id = g.inner.alloc_id();
        let sink = Rc::new(SubscribeInner {
            graph: g.inner.clone(),
            id,
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
    }
}
