use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::super::DropResource;
use super::{
    NodeId,
    edges::ParentDiff,
    node_hash::{NodeMap, NodeSet},
};

/// `when_connect` lifecycle: connect while a node has children, drop the resource when not.
///
/// `apply` / `register` / `unregister` only mark the node pending. The graph runs
/// [`Self::flush`] after the wave and after `on_after_transaction`, so installing and
/// tearing down external handlers is not part of propagation.
pub(super) struct Watch {
    connect: RefCell<NodeMap<Rc<dyn Fn() -> DropResource>>>,
    connected: RefCell<NodeMap<DropResource>>,
    pending: RefCell<NodeSet>,
    flushing: Cell<bool>,
}

/// Clears `flushing` when the flush ends (including panic).
struct Flushing<'a> {
    watch: &'a Watch,
}

impl Drop for Flushing<'_> {
    fn drop(&mut self) {
        self.watch.flushing.set(false);
    }
}

impl Watch {
    pub(super) fn new() -> Self {
        Self {
            connect: RefCell::new(NodeMap::default()),
            connected: RefCell::new(NodeMap::default()),
            pending: RefCell::new(NodeSet::default()),
            flushing: Cell::new(false),
        }
    }

    pub(super) fn register(
        &self,
        id: NodeId,
        connect: Rc<dyn Fn() -> DropResource>,
        watched: bool,
    ) {
        self.connect.borrow_mut().insert(id, connect);
        if watched {
            self.schedule(id);
        }
    }

    pub(super) fn unregister(&self, id: NodeId) {
        self.connect.borrow_mut().remove(&id);
        self.schedule(id);
    }

    pub(super) fn apply(&self, diff: ParentDiff) {
        for id in diff.became_watched {
            self.schedule(id);
        }
        for id in diff.became_unwatched {
            self.schedule(id);
        }
    }

    pub(super) fn on_unwatched_many(&self, ids: Vec<NodeId>) {
        for id in ids {
            self.schedule(id);
        }
    }

    fn schedule(&self, id: NodeId) {
        self.pending.borrow_mut().insert(id);
    }

    /// Match connectedness to `is_watched`. No-op when a node was watched and unwatched
    /// before this flush (net unchanged, never connected).
    ///
    /// Reentrant calls return at once and leave their work in `pending`, for the loop
    /// below to pick up. A `connect` closure may write, and a write runs a whole wave -
    /// transaction, propagation, and another flush - before the closure returns. Letting
    /// that inner flush run would judge this node while its resource is not in
    /// `connected` yet: a node unwatched by its own connect would keep the resource
    /// forever, and one re-watched there would connect twice, the second resource
    /// silently replacing the first.
    ///
    /// Disconnect only tears down the external handler. `Value::set` from that `Drop` is
    /// refused, so a disconnect cannot watch its node again.
    ///
    /// A `create` may write, though, and one node's write can watch another - which is how
    /// a chain of connects works, and also how a ring of them could hand the connection
    /// round forever, each write undoing the one before. So a flush is **one round of
    /// connect decisions per node**: a node connects at most once here. A legitimate chain
    /// connects each of its nodes exactly once however long it is, so only a ring reaches
    /// the limit. Reaching it leaves that node disconnected, and its id is returned for the
    /// caller to report - `Watch` has no logger of its own.
    pub(super) fn flush(&self, is_watched: impl Fn(NodeId) -> bool) -> Vec<NodeId> {
        if self.flushing.get() {
            return Vec::new();
        }
        self.flushing.set(true);
        let _guard = Flushing { watch: self };

        let mut connected_once = NodeSet::default();
        let mut left_disconnected: Vec<NodeId> = Vec::new();

        loop {
            let mut pending: Vec<NodeId> = self.pending.borrow_mut().drain().collect();
            if pending.is_empty() {
                return left_disconnected;
            }
            // By id, so a round is processed in the order the nodes were created rather
            // than in hash order. When `create` closures fight over who is watched, the
            // order decides which of them wins and whether one has to be cut - and that
            // should be the same on every run, not a different answer per page load.
            pending.sort_unstable();
            for id in pending {
                let watched = is_watched(id);
                let has_connect = self.connect.borrow().contains_key(&id);
                let is_connected = self.connected.borrow().contains_key(&id);

                if is_connected && (!watched || !has_connect) {
                    let _dropped = self.connected.borrow_mut().remove(&id);
                }

                if watched && has_connect && !self.connected.borrow().contains_key(&id) {
                    if !connected_once.insert(id) {
                        if !left_disconnected.contains(&id) {
                            left_disconnected.push(id);
                        }
                        continue;
                    }
                    let Some(connect) = self.connect.borrow().get(&id).cloned() else {
                        continue;
                    };
                    let resource = connect();
                    self.connected.borrow_mut().insert(id, resource);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn register_connects_when_watched() {
        let watch = Watch::new();
        let n = Rc::new(Cell::new(0));
        watch.register(
            NodeId(1),
            Rc::new({
                let n = n.clone();
                move || {
                    n.set(1);
                    DropResource::new(|| {})
                }
            }),
            true,
        );
        assert_eq!(n.get(), 0, "connect waits for flush");
        watch.flush(|id| id == NodeId(1));
        assert_eq!(n.get(), 1);
    }

    fn connect_flag(flag: &Rc<Cell<i32>>) -> Rc<dyn Fn() -> DropResource> {
        let flag = flag.clone();
        Rc::new(move || {
            flag.set(flag.get() + 1);
            DropResource::new({
                let flag = flag.clone();
                move || flag.set(flag.get() - 1)
            })
        })
    }

    #[test]
    fn watched_twice_connects_once() {
        let watch = Watch::new();
        let connects = Rc::new(Cell::new(0));
        let connect = Rc::new({
            let connects = connects.clone();
            move || {
                connects.set(connects.get() + 1);
                DropResource::new(|| {})
            }
        });
        watch.register(NodeId(1), connect.clone(), true);
        watch.register(NodeId(1), connect, true);
        watch.flush(|id| id == NodeId(1));
        assert_eq!(connects.get(), 1);
    }

    #[test]
    fn unregister_drops_connected() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.flush(|id| id == NodeId(1));
        watch.unregister(NodeId(1));
        watch.flush(|_| false);
        assert_eq!(live.get(), 0);
    }

    #[test]
    fn apply_unwatched_drops_connected() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.flush(|id| id == NodeId(1));
        watch.apply(ParentDiff {
            became_watched: Vec::new(),
            became_unwatched: vec![NodeId(1)],
        });
        watch.flush(|_| false);
        assert_eq!(live.get(), 0);
    }

    #[test]
    fn unwatched_many_drops_connected() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.flush(|id| id == NodeId(1));
        watch.on_unwatched_many(vec![NodeId(1)]);
        watch.flush(|_| false);
        assert_eq!(live.get(), 0);
    }

    /// A `connect` closure that writes runs a whole wave before it returns, and that wave
    /// ends in another flush. Here the wave costs the node its last child: the resource
    /// the closure is about to return is already stale, and must not stay alive.
    #[test]
    fn connect_that_unwatches_itself_is_disconnected() {
        let watch = Rc::new(Watch::new());
        let live = Rc::new(Cell::new(0));
        let watched = Rc::new(Cell::new(true));

        let connect: Rc<dyn Fn() -> DropResource> = Rc::new({
            let watch = Rc::downgrade(&watch);
            let live = live.clone();
            let watched = watched.clone();
            move || {
                let inner = connect_flag(&live)();
                // What a write from `connect` amounts to: the node loses its last child,
                // and the wave ends by flushing again.
                watched.set(false);
                if let Some(watch) = watch.upgrade() {
                    watch.schedule(NodeId(1));
                    let watched = watched.clone();
                    watch.flush(move |_| watched.get());
                }
                inner
            }
        });

        watch.register(NodeId(1), connect, true);
        let is_watched = watched.clone();
        watch.flush(move |_| is_watched.get());

        assert_eq!(live.get(), 0, "unwatched, so it must not stay connected");
    }

    /// Connect unwatches the node; disconnect must not watch it again. The graph refuses
    /// `set` from `Drop`, so this is only a Cell here - the flush still has to end after
    /// the one disconnect.
    #[test]
    fn disconnect_does_not_reconnect() {
        let watch = Rc::new(Watch::new());
        let connects = Rc::new(Cell::new(0));
        let watched = Rc::new(Cell::new(true));

        let connect: Rc<dyn Fn() -> DropResource> = Rc::new({
            let watch = Rc::downgrade(&watch);
            let connects = connects.clone();
            let watched = watched.clone();
            move || {
                connects.set(connects.get() + 1);
                watched.set(false);
                if let Some(watch) = watch.upgrade() {
                    watch.schedule(NodeId(1));
                }
                DropResource::new(|| {})
            }
        });

        watch.register(NodeId(1), connect, true);
        let is_watched = watched.clone();
        watch.flush(move |_| is_watched.get());

        assert_eq!(connects.get(), 1);
    }

    /// A flush is one round of connect decisions per node. Here the node keeps being
    /// handed back its watcher after each disconnect - in the graph that can only come
    /// from another node's `create`, so a `Cell` stands in for it. The second connect is
    /// refused, the node is left disconnected, and its id is handed back to be reported.
    #[test]
    fn a_node_connects_at_most_once_per_flush() {
        let watch = Rc::new(Watch::new());
        let connects = Rc::new(Cell::new(0));
        let watched = Rc::new(Cell::new(true));

        let connect: Rc<dyn Fn() -> DropResource> = Rc::new({
            let watch = Rc::downgrade(&watch);
            let connects = connects.clone();
            let watched = watched.clone();
            move || {
                connects.set(connects.get() + 1);
                watched.set(false);
                if let Some(watch) = watch.upgrade() {
                    watch.schedule(NodeId(1));
                }
                DropResource::new({
                    let watch = watch.clone();
                    let watched = watched.clone();
                    move || {
                        watched.set(true);
                        if let Some(watch) = watch.upgrade() {
                            watch.schedule(NodeId(1));
                        }
                    }
                })
            }
        });

        watch.register(NodeId(1), connect, true);
        let is_watched = watched.clone();
        let left_disconnected = watch.flush(move |_| is_watched.get());

        assert_eq!(connects.get(), 1);
        assert_eq!(left_disconnected, vec![NodeId(1)]);
    }

    /// A chain - connecting one node watches the next - must run to the end.
    #[test]
    fn a_chain_of_connects_is_not_a_loop() {
        let watch = Rc::new(Watch::new());
        let reached = Rc::new(Cell::new(0u64));
        let length = 30u64;

        for id in 1..=length {
            let connect: Rc<dyn Fn() -> DropResource> = Rc::new({
                let watch = Rc::downgrade(&watch);
                let reached = reached.clone();
                move || {
                    reached.set(id);
                    if let Some(watch) = watch.upgrade() {
                        watch.schedule(NodeId(id + 1));
                    }
                    DropResource::new(|| {})
                }
            });
            watch.register(NodeId(id), connect, id == 1);
        }

        watch.flush(|_| true);

        assert_eq!(reached.get(), length);
    }

    /// Same reentrancy, but the node is still watched afterwards: the inner flush must not
    /// connect a second time. Without the guard this recurses until the stack runs out.
    #[test]
    fn reentrant_flush_does_not_connect_twice() {
        let watch = Rc::new(Watch::new());
        let connects = Rc::new(Cell::new(0));

        let connect: Rc<dyn Fn() -> DropResource> = Rc::new({
            let watch = Rc::downgrade(&watch);
            let connects = connects.clone();
            move || {
                connects.set(connects.get() + 1);
                if let Some(watch) = watch.upgrade() {
                    watch.schedule(NodeId(1));
                    watch.flush(|_| true);
                }
                DropResource::new(|| {})
            }
        });

        watch.register(NodeId(1), connect, true);
        watch.flush(|_| true);

        assert_eq!(connects.get(), 1);
    }

    #[test]
    fn watch_and_unwatch_before_flush_does_not_connect() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.apply(ParentDiff {
            became_watched: Vec::new(),
            became_unwatched: vec![NodeId(1)],
        });
        watch.flush(|_| false);
        assert_eq!(live.get(), 0);
    }
}
