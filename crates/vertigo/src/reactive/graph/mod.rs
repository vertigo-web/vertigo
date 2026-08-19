use std::{cell::Cell, rc::Rc};

use super::{Computed, Context, DropResource, Value, context::ParentList};

mod dirty;
mod edges;
mod hooks;
mod nodes;
mod watch;

use dirty::Dirty;
use edges::Edges;
use hooks::Hooks;
use nodes::Nodes;
use watch::Watch;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) struct NodeId(pub u64);

/// Identity of a [`Value`] or [`Computed`] node.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct GraphId(u64);

impl GraphId {
    pub fn id(&self) -> u64 {
        self.0
    }

    pub(crate) fn from_node(id: NodeId) -> Self {
        GraphId(id.0)
    }
}

pub(crate) trait ErasedNode {
    fn refresh(&self) -> bool;
}

/// One reactive graph. Nodes created from different graphs do not see each other.
pub struct Graph {
    pub(crate) inner: Rc<GraphInner>,
}

/// Orchestrator: transactions, `propagate`, `set_parents`, `unregister`.
///
/// Persistent state lives in `Nodes`, `Edges`, `Dirty`, `Watch`, `Hooks`.
pub(crate) struct GraphInner {
    next_id: Cell<u64>,
    tx: Cell<u32>,
    propagating: Cell<bool>,
    nodes: Nodes,
    edges: Edges,
    dirty: Dirty,
    watch: Watch,
    hooks: Hooks,
}

impl Clone for Graph {
    fn clone(&self) -> Self {
        Graph {
            inner: self.inner.clone(),
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            inner: Rc::new(GraphInner {
                next_id: Cell::new(1),
                tx: Cell::new(0),
                propagating: Cell::new(false),
                nodes: Nodes::new(),
                edges: Edges::new(),
                dirty: Dirty::new(),
                watch: Watch::new(),
                hooks: Hooks::new(),
            }),
        }
    }

    pub fn value<T: Clone + PartialEq + 'static>(&self, value: T) -> Value<T> {
        Value::create(self.inner.clone(), value)
    }

    pub fn computed<T: Clone + PartialEq + 'static>(
        &self,
        compute: impl Fn(&Context) -> T + 'static,
    ) -> Computed<T> {
        Computed::create(self.inner.clone(), compute)
    }

    pub fn transaction<R>(&self, f: impl FnOnce(&Context) -> R) -> R {
        self.inner.tx.set(self.inner.tx.get() + 1);
        let ctx = Context::read();
        let result = f(&ctx);
        self.inner.tx.set(self.inner.tx.get() - 1);
        if self.inner.tx.get() == 0 {
            let nested_in_propagate = self.inner.propagating.get();
            self.inner.propagate();
            if !nested_in_propagate {
                self.inner.hooks.fire();
            }
        }
        result
    }

    pub fn on_after_transaction(&self, callback: impl Fn() + 'static) -> DropResource {
        let id = self.inner.hooks.insert(callback);
        let inner = self.inner.clone();
        DropResource::new(move || {
            inner.hooks.remove(id);
        })
    }
}

impl GraphInner {
    pub(crate) fn alloc_id(&self) -> NodeId {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        NodeId(id)
    }

    pub(crate) fn register(&self, id: NodeId, slot: Rc<dyn ErasedNode>) {
        self.nodes.register(id, slot);
    }

    pub(crate) fn enqueue(&self, id: NodeId) {
        self.dirty.enqueue(id, &self.edges);
    }

    pub(crate) fn register_connect(&self, id: NodeId, connect: Rc<dyn Fn() -> DropResource>) {
        self.watch.register(id, connect, self.edges.is_watched(id));
    }

    pub(crate) fn set_parents(&self, child: NodeId, pairs: ParentList) {
        if let Some(diff) = self.edges.replace(child, pairs) {
            self.watch.apply(diff);
        }
    }

    pub(crate) fn unregister(&self, id: NodeId) {
        let was_dirty = self.dirty.contains(id);
        self.dirty.dequeue(id);
        if was_dirty {
            self.dirty.release_parent(id, &self.edges);
        }
        self.nodes.remove(id);
        self.watch.unregister(id);
        let became_unwatched = self.edges.unregister(id);
        self.watch.on_unwatched_many(became_unwatched);
    }

    /// Process dirty nodes in topological order. Dependents are enqueued only when
    /// the node's value changed.
    pub(crate) fn propagate(&self) {
        if self.tx.get() != 0 || self.propagating.get() {
            return;
        }
        self.propagating.set(true);

        loop {
            let Some(id) = self.dirty.take_ready() else {
                if let Some(leftover) = self.dirty.cycle_leftover() {
                    self.propagating.set(false);
                    panic!("vertigo: cycle in dirty graph ({leftover:?})");
                }
                break;
            };

            self.dirty.dequeue(id);

            let Some(node) = self.nodes.upgrade(id) else {
                self.dirty.release_parent(id, &self.edges);
                continue;
            };

            let changed = node.refresh();
            self.dirty.after_refresh(id, changed, &self.edges);
        }

        self.propagating.set(false);
    }
}
