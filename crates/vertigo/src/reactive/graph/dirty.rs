use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
};

use super::{NodeId, edges::Edges};

/// Kahn-style worklist for one propagation wave.
///
/// A dirty node is **ready** when none of its parents are still dirty. `dirty_parent_count`
/// stores that remaining count; `0` (or missing) means the node can be processed.
/// `scratch_children` is reused so `propagate` does not allocate a new `Vec` per node.
pub(super) struct Dirty {
    in_dirty: RefCell<HashSet<NodeId>>,
    dirty_parent_count: RefCell<HashMap<NodeId, u32>>,
    ready: RefCell<VecDeque<NodeId>>,
    scratch_children: RefCell<Vec<NodeId>>,
}

impl Dirty {
    pub(super) fn new() -> Self {
        Self {
            in_dirty: RefCell::new(HashSet::new()),
            dirty_parent_count: RefCell::new(HashMap::new()),
            ready: RefCell::new(VecDeque::new()),
            scratch_children: RefCell::new(Vec::new()),
        }
    }

    pub(super) fn contains(&self, id: NodeId) -> bool {
        self.in_dirty.borrow().contains(&id)
    }

    pub(super) fn enqueue(&self, id: NodeId, edges: &Edges) {
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
        while let Some(id) = ready.pop_front() {
            if in_dirty.contains(&id) {
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
    pub(super) fn release_parent(&self, parent: NodeId, edges: &Edges) {
        if self.dirty_parent_count.borrow().is_empty() {
            return;
        }
        self.fill_scratch(parent, edges);
        self.release_from_scratch();
    }

    /// After `refresh`: release waiting children; enqueue dependents only on value change.
    pub(super) fn after_refresh(&self, id: NodeId, changed: bool, edges: &Edges) {
        let need_release = !self.dirty_parent_count.borrow().is_empty();
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
        {
            let children = self.scratch_children.borrow();
            let mut counts = self.dirty_parent_count.borrow_mut();
            for child in children.iter() {
                if let Some(count) = counts.get_mut(child) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        newly_ready.push(*child);
                    }
                }
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
}
