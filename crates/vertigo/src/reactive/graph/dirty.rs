use std::{cell::RefCell, collections::VecDeque};

use super::{
    NodeId,
    edges::Edges,
    node_hash::{NodeMap, NodeSet},
};

/// Kahn-style worklist for one propagation wave.
///
/// Children are enqueued only when a parent’s value changed (equality cutoff). A join
/// node that runs before a later parent is therefore pulled to freshness in `get`, not
/// by marking the whole descendant set up front.
///
/// A dirty node is **ready** when none of its parents are still dirty. `dirty_parent_count`
/// stores that remaining count; `0` (or missing) means the node can be processed.
/// `scratch_children` is reused so `propagate` does not allocate a new `Vec` per node.
pub(super) struct Dirty {
    in_dirty: RefCell<NodeSet>,
    dirty_parent_count: RefCell<NodeMap<u32>>,
    ready: RefCell<VecDeque<NodeId>>,
    scratch_children: RefCell<Vec<NodeId>>,
    /// Already refreshed (or confirmed fresh) in this wave. At most one `refresh` per id.
    done: RefCell<NodeSet>,
    /// Subset of `done` whose value changed.
    changed: RefCell<NodeSet>,
    /// Nodes whose `refresh` is on the stack (gray). Re-entering one is a cycle.
    refreshing: RefCell<NodeSet>,
    /// The same nodes in the order they were entered, so a cycle can name its path.
    refresh_stack: RefCell<Vec<NodeId>>,
}

/// Clears the gray mark when `refresh` returns, including panic.
pub(super) struct Refreshing<'a> {
    dirty: &'a Dirty,
    id: NodeId,
}

impl Drop for Refreshing<'_> {
    fn drop(&mut self) {
        self.dirty.refreshing.borrow_mut().remove(&self.id);
        self.dirty.refresh_stack.borrow_mut().pop();
    }
}

impl Dirty {
    pub(super) fn new() -> Self {
        Self {
            in_dirty: RefCell::new(NodeSet::default()),
            dirty_parent_count: RefCell::new(NodeMap::default()),
            ready: RefCell::new(VecDeque::new()),
            scratch_children: RefCell::new(Vec::new()),
            done: RefCell::new(NodeSet::default()),
            changed: RefCell::new(NodeSet::default()),
            refreshing: RefCell::new(NodeSet::default()),
            refresh_stack: RefCell::new(Vec::new()),
        }
    }

    pub(super) fn begin_wave(&self) {
        self.done.borrow_mut().clear();
        self.changed.borrow_mut().clear();
    }

    pub(super) fn contains(&self, id: NodeId) -> bool {
        self.in_dirty.borrow().contains(&id)
    }

    pub(super) fn is_done(&self, id: NodeId) -> bool {
        self.done.borrow().contains(&id)
    }

    pub(super) fn changed_this_wave(&self, id: NodeId) -> bool {
        self.changed.borrow().contains(&id)
    }

    pub(super) fn is_refreshing(&self, id: NodeId) -> bool {
        self.refreshing.borrow().contains(&id)
    }

    /// Mark `id` gray for the duration of `refresh`.
    ///
    /// A node already gray means a cycle, but the read that closes it is caught earlier,
    /// in `ensure_fresh`, where the path is still on the stack - hence an assertion here
    /// rather than a second panic with a poorer message.
    pub(super) fn enter_refresh(&self, id: NodeId) -> Refreshing<'_> {
        let marked = self.refreshing.borrow_mut().insert(id);
        debug_assert!(marked, "{id:?} is already refreshing");
        self.refresh_stack.borrow_mut().push(id);
        Refreshing { dirty: self, id }
    }

    /// The refresh path from `id` back to itself, as `NodeId(1) -> NodeId(2) -> NodeId(1)`.
    pub(super) fn cycle_path(&self, id: NodeId) -> String {
        let stack = self.refresh_stack.borrow();
        let start = stack.iter().position(|entry| *entry == id).unwrap_or(0);
        let mut path: Vec<String> = stack[start..].iter().map(|n| format!("{n:?}")).collect();
        path.push(format!("{id:?}"));
        path.join(" -> ")
    }

    pub(super) fn finish(&self, id: NodeId, changed: bool) {
        self.done.borrow_mut().insert(id);
        if changed {
            self.changed.borrow_mut().insert(id);
        }
    }

    pub(super) fn enqueue(&self, id: NodeId, edges: &Edges) {
        if self.done.borrow().contains(&id) || self.refreshing.borrow().contains(&id) {
            return;
        }
        if !self.in_dirty.borrow_mut().insert(id) {
            return;
        }
        let count = self.count_dirty_parents(id, edges);
        if count == 0 {
            self.ready.borrow_mut().push_back(id);
        } else {
            self.dirty_parent_count.borrow_mut().insert(id, count);
        }
    }

    pub(super) fn take_ready(&self) -> Option<NodeId> {
        let mut ready = self.ready.borrow_mut();
        let in_dirty = self.in_dirty.borrow();
        let refreshing = self.refreshing.borrow();
        let done = self.done.borrow();
        while let Some(id) = ready.pop_front() {
            if in_dirty.contains(&id) && !refreshing.contains(&id) && !done.contains(&id) {
                return Some(id);
            }
        }
        None
    }

    pub(super) fn dequeue(&self, id: NodeId) {
        self.in_dirty.borrow_mut().remove(&id);
        self.dirty_parent_count.borrow_mut().remove(&id);
    }

    /// `Some` leftover ids when `ready` is empty but dirty nodes remain (a cycle).
    pub(super) fn cycle_leftover(&self) -> Option<Vec<NodeId>> {
        let in_dirty = self.in_dirty.borrow();
        if in_dirty.is_empty() {
            None
        } else {
            Some(in_dirty.iter().copied().collect())
        }
    }

    /// Parent left the dirty set: dependents waiting on it may become ready.
    ///
    /// Empty `dirty_parent_count` means nobody is waiting, so skip copying children.
    pub(super) fn release_parent(&self, parent: NodeId, edges: &Edges) {
        if self.dirty_parent_count.borrow().is_empty() {
            return;
        }
        self.fill_scratch(parent, edges);
        self.release_from_scratch();
    }

    /// After `refresh` of a node that was in the dirty set: release the children waiting
    /// on it, and enqueue dependents if its value changed.
    ///
    /// Cutoff with no waiters returns before `fill_scratch` so a large fan-out is not copied.
    pub(super) fn after_refresh(&self, id: NodeId, changed: bool, edges: &Edges) {
        self.settle(id, changed, true, edges);
    }

    /// After `refresh` of a node pulled by a `get`: it was never dirty, so no child ever
    /// counted it among the parents it is waiting for, and there is nothing to release.
    /// Releasing anyway would zero a child's count while a parent that really is dirty
    /// has not run - the child would be handed out early and have to pull that parent.
    pub(super) fn after_pull(&self, id: NodeId, changed: bool, edges: &Edges) {
        self.settle(id, changed, false, edges);
    }

    fn settle(&self, id: NodeId, changed: bool, was_dirty: bool, edges: &Edges) {
        let need_release = was_dirty && !self.dirty_parent_count.borrow().is_empty();
        if !need_release && !changed {
            return;
        }

        self.fill_scratch(id, edges);
        if need_release {
            self.release_from_scratch();
        }
        if changed {
            self.enqueue_from_scratch(edges);
        }
    }

    fn count_dirty_parents(&self, id: NodeId, edges: &Edges) -> u32 {
        let in_dirty = self.in_dirty.borrow();
        edges.count_parents_if(id, |parent| in_dirty.contains(&parent))
    }

    fn fill_scratch(&self, id: NodeId, edges: &Edges) {
        let mut buf = self.scratch_children.borrow_mut();
        edges.copy_children(id, &mut buf);
    }

    fn release_from_scratch(&self) {
        let mut newly_ready = Vec::new();
        let mut zeroed = Vec::new();
        {
            let children = self.scratch_children.borrow();
            let mut counts = self.dirty_parent_count.borrow_mut();
            let refreshing = self.refreshing.borrow();
            let done = self.done.borrow();
            for child in children.iter() {
                if let Some(count) = counts.get_mut(child) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        zeroed.push(*child);
                        if !refreshing.contains(child) && !done.contains(child) {
                            newly_ready.push(*child);
                        }
                    }
                }
            }
            for id in zeroed {
                counts.remove(&id);
            }
        }
        if !newly_ready.is_empty() {
            self.ready.borrow_mut().extend(newly_ready);
        }
    }

    fn enqueue_from_scratch(&self, edges: &Edges) {
        let children = self.scratch_children.borrow();
        for child in children.iter() {
            self.enqueue(*child, edges);
        }
    }

    #[cfg(test)]
    fn wait_count(&self, id: NodeId) -> Option<u32> {
        self.dirty_parent_count.borrow().get(&id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ErasedNode, NodeId, edges::Edges};
    use super::*;
    use std::rc::Rc;

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
    fn ready_is_fifo() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(2), &edges);
        assert_eq!(dirty.take_ready(), Some(NodeId(1)));
        assert_eq!(dirty.take_ready(), Some(NodeId(2)));
    }

    #[test]
    fn leftover_when_waiting_child_never_released() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(2), &edges);
        dirty.dequeue(NodeId(1));
        assert_eq!(dirty.take_ready(), None);
        assert!(dirty.cycle_leftover().is_some());
    }

    #[test]
    fn enqueue_dedups_ready() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(1), &edges);
        assert_eq!(dirty.take_ready(), Some(NodeId(1)));
        assert_eq!(dirty.take_ready(), None);
    }

    #[test]
    fn child_not_ready_while_parent_dirty() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(2), &edges);
        assert_eq!(dirty.take_ready(), Some(NodeId(1)));
        assert_eq!(dirty.take_ready(), None);
    }

    #[test]
    fn take_ready_skips_dequeued() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        dirty.enqueue(NodeId(1), &edges);
        dirty.dequeue(NodeId(1));
        assert_eq!(dirty.take_ready(), None);
    }

    #[test]
    fn dequeue_clears_wait_count() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(2), &edges);
        dirty.dequeue(NodeId(2));
        assert_eq!(dirty.wait_count(NodeId(2)), None);
    }

    #[test]
    fn cutoff_does_not_enqueue_waiting_sibling() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        edges.replace(NodeId(3), vec![slot(1)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(3), &edges);
        dirty.dequeue(NodeId(1));
        dirty.after_refresh(NodeId(1), false, &edges);
        assert!(!dirty.contains(NodeId(2)));
    }

    #[test]
    fn cutoff_does_not_copy_or_enqueue_children() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.dequeue(NodeId(1));
        dirty.after_refresh(NodeId(1), false, &edges);
        assert!(!dirty.contains(NodeId(2)));
        assert_eq!(dirty.take_ready(), None);
    }

    #[test]
    fn child_waits_for_both_dirty_parents() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        edges.replace(NodeId(3), vec![slot(1), slot(2)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(2), &edges);
        dirty.enqueue(NodeId(3), &edges);
        assert_eq!(dirty.wait_count(NodeId(3)), Some(2));

        let Some(first) = dirty.take_ready() else {
            panic!("one of the two parents must be ready");
        };
        assert!(first == NodeId(1) || first == NodeId(2));
        dirty.dequeue(first);
        dirty.after_refresh(first, true, &edges);

        let Some(second) = dirty.take_ready() else {
            panic!("the other parent must be ready");
        };
        assert!(second == NodeId(1) || second == NodeId(2));
        assert_ne!(second, first);
        dirty.dequeue(second);
        dirty.after_refresh(second, true, &edges);
        assert_eq!(dirty.take_ready(), Some(NodeId(3)));
    }

    /// A node pulled by a read was never dirty, so no child ever counted it as a dirty
    /// parent. Refreshing it must not release children that are waiting for parents which
    /// really are dirty.
    #[test]
    fn a_pulled_node_does_not_release_waiting_children() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        // `3` reads two parents that are dirty and one that is not.
        edges.replace(NodeId(3), vec![slot(1), slot(2), slot(4)]);
        dirty.enqueue(NodeId(1), &edges);
        dirty.enqueue(NodeId(2), &edges);
        dirty.enqueue(NodeId(3), &edges);
        assert_eq!(dirty.wait_count(NodeId(3)), Some(2));

        dirty.after_pull(NodeId(4), true, &edges);

        assert_eq!(dirty.wait_count(NodeId(3)), Some(2));
        assert_eq!(dirty.take_ready(), Some(NodeId(1)));
    }

    #[test]
    fn begin_wave_allows_enqueue_of_previously_done_node() {
        let dirty = Dirty::new();
        let edges = Edges::new();
        dirty.finish(NodeId(1), true);
        dirty.enqueue(NodeId(1), &edges);
        assert!(!dirty.contains(NodeId(1)));
        dirty.begin_wave();
        dirty.enqueue(NodeId(1), &edges);
        assert!(dirty.contains(NodeId(1)));
    }

    #[test]
    fn enter_refresh_is_gray() {
        let dirty = Dirty::new();
        let _guard = dirty.enter_refresh(NodeId(1));
        assert!(dirty.is_refreshing(NodeId(1)));
    }

    /// The read that closes a cycle is caught in `ensure_fresh`; this is the backstop.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "already refreshing")]
    fn reentering_refresh_is_rejected() {
        let dirty = Dirty::new();
        let _guard = dirty.enter_refresh(NodeId(1));
        let _again = dirty.enter_refresh(NodeId(1));
    }

    #[test]
    fn cycle_path_names_the_way_back() {
        let dirty = Dirty::new();
        let _outer = dirty.enter_refresh(NodeId(7));
        let _a = dirty.enter_refresh(NodeId(1));
        let _b = dirty.enter_refresh(NodeId(2));

        assert_eq!(
            dirty.cycle_path(NodeId(1)),
            "NodeId(1) -> NodeId(2) -> NodeId(1)"
        );
    }
}
