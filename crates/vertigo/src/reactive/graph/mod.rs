use std::{cell::Cell, rc::Rc};

use super::{Computed, Context, DropResource, Value, context::ParentList};

mod dirty;
mod edges;
mod hooks;
mod nodes;
mod transaction;
mod watch;

use dirty::Dirty;
use edges::Edges;
use hooks::Hooks;
use nodes::Nodes;
use transaction::Transaction;
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
    tx: Transaction,
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
                tx: Transaction::new(),
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
        self.inner.tx.enter();
        let ctx = Context::read();
        let result = f(&ctx);
        if let Some(leave) = self.inner.tx.leave() {
            self.inner.propagate();
            if !leave.already_propagating {
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
        if !self.tx.can_propagate() {
            return;
        }
        let _guard = self.tx.start_propagate();

        loop {
            let Some(id) = self.dirty.take_ready() else {
                if let Some(leftover) = self.dirty.cycle_leftover() {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct N;
    impl ErasedNode for N {
        fn refresh(&self) -> bool {
            false
        }
    }

    fn slot(id: u64) -> (NodeId, Rc<dyn ErasedNode>) {
        (NodeId(id), Rc::new(N))
    }

    #[test]
    fn unregister_releases_waiting_child() {
        let g = Graph::new();
        let inner = &*g.inner;
        inner.edges.replace(NodeId(2), vec![slot(1)]);
        inner.dirty.enqueue(NodeId(1), &inner.edges);
        inner.dirty.enqueue(NodeId(2), &inner.edges);
        inner.unregister(NodeId(1));
        assert_eq!(inner.dirty.take_ready(), Some(NodeId(2)));
    }

    #[test]
    fn dead_dirty_parent_releases_waiting_child() {
        let g = Graph::new();
        let inner = &*g.inner;
        inner.edges.replace(NodeId(2), vec![slot(1)]);
        inner.dirty.enqueue(NodeId(1), &inner.edges);
        inner.dirty.enqueue(NodeId(2), &inner.edges);
        inner.propagate();
        assert!(!inner.dirty.contains(NodeId(2)));
    }
}
