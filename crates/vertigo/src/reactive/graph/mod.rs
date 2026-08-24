use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{Computed, Context, DropResource, Value, context::ParentList};

mod dirty;
mod edges;
mod hooks;
mod logger;
mod nodes;
mod transaction;
mod watch;

use dirty::Dirty;
use edges::Edges;
use hooks::Hooks;
use nodes::Nodes;
pub(crate) use transaction::CallbackGuard;
use transaction::Transaction;
use watch::Watch;

pub(crate) use logger::Logger;

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

pub(crate) const BLOCKED_WRITE: &str = "vertigo: Value::set is not allowed from a computed, a subscribe callback, a DropResource, or during propagation";

/// Parent buffers kept for reuse by `parents_changed`; see there.
const MAX_POOLED_PARENT_BUFFERS: usize = 64;

pub(crate) const CYCLE: &str = "vertigo: cycle in the reactive graph";

pub(crate) const CONNECT_LOOP: &str = "vertigo: when_connect closures are undoing each other - a node cannot connect twice in one flush, so it is left disconnected";

/// One reactive graph. Nodes created from different graphs do not see each other.
pub struct Graph {
    pub(crate) inner: Rc<GraphInner>,
}

/// Orchestrator: transactions, `propagate`, `set_parents`, `unregister`.
///
/// Persistent state lives in `Nodes`, `Edges`, `Dirty`, `Watch`, `Hooks`, `Logger`.
pub(crate) struct GraphInner {
    next_id: Cell<u64>,
    tx: Transaction,
    nodes: Nodes,
    edges: Edges,
    dirty: Dirty,
    watch: Watch,
    hooks: Hooks,
    logger: Logger,
    /// Buffers for `parents_changed`, which recurses and so needs one per level.
    parent_scratch: RefCell<Vec<Vec<NodeId>>>,
    /// How many nodes `parents_changed` has walked. Pins the cost of a pull - see
    /// `ensure_fresh`, where a node found fresh is recorded so it is walked only once.
    #[cfg(test)]
    parent_walks: Cell<u64>,
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
                logger: Logger::new(),
                parent_scratch: RefCell::new(Vec::new()),
                #[cfg(test)]
                parent_walks: Cell::new(0),
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
        // A transaction opened from inside a wave (`subscribe`, `Value::change`, or any
        // render running in a callback) is outermost by depth but is not a new wave, and
        // must leave the running wave's bookkeeping alone.
        let outermost = self.inner.tx.enter();
        if outermost && !self.inner.tx.is_propagating() {
            self.inner.dirty.begin_wave();
        }
        let ctx = Context::read();
        let result = f(&ctx);
        if let Some(leave) = self.inner.tx.leave() {
            self.inner.propagate();
            if !leave.already_propagating {
                self.inner.hooks.fire();
                self.inner.flush_watch();
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

    /// Test facility: see [`Logger`].
    #[cfg(test)]
    pub(crate) fn logger(&self) -> Logger {
        self.inner.logger.clone()
    }

    /// Nodes walked by `parents_changed` so far, and reset the count.
    #[cfg(test)]
    pub(crate) fn take_parent_walks(&self) -> u64 {
        self.inner.parent_walks.replace(0)
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
        self.flush_watch_if_idle();
    }

    pub(crate) fn enter_callback(&self) -> CallbackGuard<'_> {
        self.tx.enter_callback()
    }

    pub(crate) fn check_write_allowed(&self) -> bool {
        if self.tx.writes_blocked() || super::drop_resource::in_drop() {
            self.logger.error(BLOCKED_WRITE);
            return false;
        }
        true
    }

    /// During a wave, make `id` current before a `get` returns its cache.
    ///
    /// Dirty nodes are refreshed now. A node that is not dirty may still be stale when
    /// an ancestor changed; parents are pulled first, and this node refreshes only if
    /// one of them changed. Unchanged fan-out is never marked dirty (equality cutoff).
    ///
    /// A node found fresh is recorded as done for the rest of the wave, so a later `get`
    /// stops here instead of walking the ancestors again. That is what keeps the cost
    /// linear: without it, every read repeats the walk, and a re-convergent graph repeats
    /// it once per path. Nothing can make such a node stale later in the same wave -
    /// sources do not change during a wave (writes from compute, subscribe and Drop are
    /// refused, `when_connect` runs after), so every ancestor is settled by the time the
    /// walk returns.
    pub(crate) fn ensure_fresh(&self, id: NodeId) {
        if !self.tx.is_propagating() || self.dirty.is_done(id) {
            return;
        }
        if self.dirty.is_refreshing(id) {
            panic!(
                "{CYCLE} - a computed read a value that depends on it: {}",
                self.dirty.cycle_path(id)
            );
        }
        if self.dirty.contains(id) {
            self.refresh_now(id);
            return;
        }
        if self.parents_changed(id) {
            self.refresh_now(id);
        } else {
            self.dirty.finish(id, false);
        }
    }

    /// Parent sets have to be copied out: the walk below re-enters the graph, so no
    /// borrow of `Edges` may be held across it.
    ///
    /// The walk recurses, so one shared buffer would not do - each level holds its own
    /// while it descends. They come from a pool instead, which turns one allocation per
    /// *node* into one per *level*, and only until the pool has grown to the depth in use.
    fn parents_changed(&self, id: NodeId) -> bool {
        #[cfg(test)]
        self.parent_walks.set(self.parent_walks.get() + 1);

        let mut parents = self.parent_scratch.borrow_mut().pop().unwrap_or_default();
        self.edges.copy_parents(id, &mut parents);

        let mut changed = false;
        for parent in parents.iter().copied() {
            self.ensure_fresh(parent);
            if self.dirty.changed_this_wave(parent) {
                changed = true;
            }
        }

        parents.clear();
        // Capped: an unusually deep walk should not leave a buffer per level behind for
        // the lifetime of the graph.
        let mut pool = self.parent_scratch.borrow_mut();
        if pool.len() < MAX_POOLED_PARENT_BUFFERS {
            pool.push(parents);
        }
        changed
    }

    fn refresh_now(&self, id: NodeId) {
        if self.dirty.is_done(id) {
            return;
        }
        let _guard = self.dirty.enter_refresh(id);

        let Some(node) = self.nodes.upgrade(id) else {
            if self.dirty.contains(id) {
                self.dirty.dequeue(id);
                self.dirty.after_refresh(id, false, &self.edges);
            }
            self.dirty.finish(id, false);
            return;
        };

        let changed = node.refresh();

        // Still the state it had before the refresh: `enqueue` refuses a node that is
        // being refreshed, so nothing could have made it dirty in between.
        //
        // A pull normally arrives here with the node already dirty: reaching it means a
        // parent changed, and a parent that changed enqueues its children before its
        // dependents can read it. The other branch is a guard, not a hot path - what it
        // must never do is release children that are waiting on parents which really are
        // dirty, for a node no child was ever waiting on.
        let was_dirty = self.dirty.contains(id);
        if was_dirty {
            self.dirty.dequeue(id);
            self.dirty.after_refresh(id, changed, &self.edges);
        } else {
            self.dirty.after_pull(id, changed, &self.edges);
        }
        self.dirty.finish(id, changed);
    }

    fn flush_watch(&self) {
        let left_disconnected = self.watch.flush(|id| self.edges.is_watched(id));
        for id in left_disconnected {
            self.logger.error(&format!("{CONNECT_LOOP} ({id:?})"));
        }
    }

    fn flush_watch_if_idle(&self) {
        if self.tx.can_propagate() {
            self.flush_watch();
        }
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
        self.flush_watch_if_idle();
    }

    /// Process dirty nodes in topological order. A node refreshes at most once per wave.
    ///
    /// Children are enqueued only when a parent’s value changed. `get` pulls a stale
    /// ancestor (dirty, or a parent that changed this wave) before returning the cache,
    /// so a join still sees every branch. Dependents of an unchanged node are not marked.
    pub(crate) fn propagate(&self) {
        if !self.tx.can_propagate() {
            return;
        }
        let _guard = self.tx.start_propagate();
        self.dirty.begin_wave();

        loop {
            let Some(id) = self.dirty.take_ready() else {
                if let Some(leftover) = self.dirty.cycle_leftover() {
                    panic!("{CYCLE} - dirty nodes with none ready ({leftover:?})");
                }
                break;
            };

            self.refresh_now(id);
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
        let logs = g.logger().listen();
        let inner = &*g.inner;
        inner.edges.replace(NodeId(2), vec![slot(1)]);
        inner.dirty.enqueue(NodeId(1), &inner.edges);
        inner.dirty.enqueue(NodeId(2), &inner.edges);
        inner.unregister(NodeId(1));
        assert_eq!(inner.dirty.take_ready(), Some(NodeId(2)));
        logs.assert_eq(&[]);
    }

    #[test]
    fn dead_dirty_parent_releases_waiting_child() {
        let g = Graph::new();
        let logs = g.logger().listen();
        let inner = &*g.inner;
        inner.edges.replace(NodeId(2), vec![slot(1)]);
        inner.dirty.enqueue(NodeId(1), &inner.edges);
        inner.dirty.enqueue(NodeId(2), &inner.edges);
        inner.propagate();
        assert!(!inner.dirty.contains(NodeId(2)));
        logs.assert_eq(&[]);
    }

    fn node_id<T: Clone + PartialEq + 'static>(computed: &Computed<T>) -> NodeId {
        NodeId(computed.id().id())
    }

    /// A node found fresh is recorded as such, so a second `get` in the same wave stops
    /// at it instead of walking its ancestors again. Without this the walk is repeated
    /// per read, and re-convergent graphs pay for it exponentially.
    #[test]
    fn a_node_found_fresh_is_recorded_for_the_wave() {
        let g = Graph::new();
        let base = g.value(1i32);
        let mid = g.computed({
            let base = base.clone();
            move |ctx| base.get(ctx)
        });
        let tip = g.computed({
            let mid = mid.clone();
            move |ctx| mid.get(ctx) + 1
        });

        // `probe` is the only dirty node in the wave; `tip` and `mid` are pulled by its
        // read and turn out to be fresh.
        let trigger = g.value(0i32);
        let probe = g.computed({
            let trigger = trigger.clone();
            let tip = tip.clone();
            move |ctx| trigger.get(ctx) + tip.get(ctx)
        });
        let _sub = probe.subscribe(|_| {});

        trigger.set(1);

        assert!(g.inner.dirty.is_done(node_id(&tip)), "tip");
        assert!(g.inner.dirty.is_done(node_id(&mid)), "mid");
    }

    /// `subscribe` and `Value::change` open a transaction, and code that renders inside a
    /// `subscribe` callback does it too - all of that can happen while a wave is running.
    /// Such a transaction is not a new wave and must not reset what the wave has recorded.
    #[test]
    fn a_transaction_opened_during_a_wave_keeps_the_wave_state() {
        let g = Graph::new();
        let a = g.value(1i32);
        let first = g.computed({
            let a = a.clone();
            move |ctx| a.get(ctx) + 1
        });
        let second = g.computed({
            let first = first.clone();
            let g = g.clone();
            move |ctx| {
                let value = first.get(ctx);
                g.transaction(|_| {});
                value
            }
        });
        let _sub = second.subscribe(|_| {});

        a.set(2);

        assert!(
            g.inner.dirty.is_done(node_id(&first)),
            "refreshed before the nested transaction, so still done after it"
        );
    }
}
