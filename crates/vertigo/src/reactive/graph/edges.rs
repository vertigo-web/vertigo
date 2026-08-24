use std::{cell::RefCell, rc::Rc};

use super::super::context::ParentList;
use super::{
    ErasedNode, NodeId,
    node_hash::{NodeMap, NodeSet},
};

/// Result of replacing a child's parents: nodes that gained or lost their last child.
pub(super) struct ParentDiff {
    pub became_watched: Vec<NodeId>,
    pub became_unwatched: Vec<NodeId>,
}

/// Bidirectional DAG plus strong parent refs (so parents outlive the child that lists them).
pub(super) struct Edges {
    child_parents: RefCell<NodeMap<NodeSet>>,
    parent_children: RefCell<NodeMap<NodeSet>>,
    parent_refs: RefCell<NodeMap<Vec<Rc<dyn ErasedNode>>>>,
}

impl Edges {
    pub(super) fn new() -> Self {
        Self {
            child_parents: RefCell::new(NodeMap::default()),
            parent_children: RefCell::new(NodeMap::default()),
            parent_refs: RefCell::new(NodeMap::default()),
        }
    }

    pub(super) fn is_watched(&self, id: NodeId) -> bool {
        self.parent_children
            .borrow()
            .get(&id)
            .is_some_and(|c| !c.is_empty())
    }

    pub(super) fn count_parents_if(&self, id: NodeId, mut pred: impl FnMut(NodeId) -> bool) -> u32 {
        match self.child_parents.borrow().get(&id) {
            Some(parents) => parents
                .iter()
                .copied()
                .filter(|&parent| pred(parent))
                .count() as u32,
            None => 0,
        }
    }

    pub(super) fn copy_children(&self, id: NodeId, buf: &mut Vec<NodeId>) {
        buf.clear();
        if let Some(set) = self.parent_children.borrow().get(&id) {
            buf.extend(set.iter().copied());
        }
    }

    pub(super) fn copy_parents(&self, id: NodeId, buf: &mut Vec<NodeId>) {
        buf.clear();
        if let Some(set) = self.child_parents.borrow().get(&id) {
            buf.extend(set.iter().copied());
        }
    }

    /// Replace `child`'s parent set. `None` means the parent ids were already the same.
    pub(super) fn replace(&self, child: NodeId, pairs: ParentList) -> Option<ParentDiff> {
        // Collect the ids before comparing. Set-against-set is linear, while comparing the
        // stored set against the raw `pairs` list is quadratic - and `pairs` carries one
        // entry per `get` call, duplicates included, so it can be much longer than the set.
        let new_parents: NodeSet = pairs.iter().map(|(id, _)| *id).collect();

        {
            let child_parents = self.child_parents.borrow();
            if let Some(old) = child_parents.get(&child)
                && *old == new_parents
            {
                return None;
            }
        }

        let kept: Vec<Rc<dyn ErasedNode>> = pairs.into_iter().map(|(_, slot)| slot).collect();

        let old = self
            .child_parents
            .borrow_mut()
            .insert(child, new_parents.clone())
            .unwrap_or_default();

        let mut became_watched = Vec::new();
        let mut became_unwatched = Vec::new();

        {
            let mut parent_children = self.parent_children.borrow_mut();

            for parent in old.difference(&new_parents) {
                if let Some(children) = parent_children.get_mut(parent) {
                    children.remove(&child);
                    if children.is_empty() {
                        became_unwatched.push(*parent);
                    }
                }
            }

            for parent in new_parents.difference(&old) {
                let children = parent_children.entry(*parent).or_default();
                let was_empty = children.is_empty();
                children.insert(child);
                if was_empty {
                    became_watched.push(*parent);
                }
            }
        }

        let old_kept = self.parent_refs.borrow_mut().insert(child, kept);
        drop(old_kept);

        Some(ParentDiff {
            became_watched,
            became_unwatched,
        })
    }

    /// Strip `id` from both adjacency maps. Returns parents that lost their last child.
    pub(super) fn unregister(&self, id: NodeId) -> Vec<NodeId> {
        let _kept = self.parent_refs.borrow_mut().remove(&id);
        let parents = self
            .child_parents
            .borrow_mut()
            .remove(&id)
            .unwrap_or_default();
        let mut became_unwatched = Vec::new();
        {
            let mut parent_children = self.parent_children.borrow_mut();
            for parent in &parents {
                if let Some(children) = parent_children.get_mut(parent) {
                    children.remove(&id);
                    if children.is_empty() {
                        became_unwatched.push(*parent);
                    }
                }
            }

            if let Some(children) = parent_children.remove(&id) {
                let mut child_parents = self.child_parents.borrow_mut();
                for child in children {
                    if let Some(ps) = child_parents.get_mut(&child) {
                        ps.remove(&id);
                    }
                }
            }
        }
        became_unwatched
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
    fn is_watched_after_replace() {
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        assert!(edges.is_watched(NodeId(1)));
    }

    #[test]
    fn empty_children_is_not_watched() {
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        edges.replace(NodeId(2), vec![]);
        assert!(!edges.is_watched(NodeId(1)));
    }

    #[test]
    fn replace_reports_unwatched() {
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        let became_unwatched = edges
            .replace(NodeId(2), vec![])
            .map(|diff| diff.became_unwatched);
        assert_eq!(became_unwatched, Some(vec![NodeId(1)]));
    }

    /// Parent sets are compared as sets: order must not matter.
    #[test]
    fn replace_with_reordered_parents_is_noop() {
        let edges = Edges::new();
        edges.replace(NodeId(3), vec![slot(1), slot(2)]);
        assert!(edges.replace(NodeId(3), vec![slot(2), slot(1)]).is_none());
    }

    /// `ParentList` holds one entry per `get` call, so the same parent can repeat.
    #[test]
    fn replace_with_duplicate_parents_is_noop() {
        let edges = Edges::new();
        edges.replace(NodeId(3), vec![slot(1)]);
        assert!(
            edges
                .replace(NodeId(3), vec![slot(1), slot(1), slot(1)])
                .is_none()
        );
    }

    #[test]
    fn replace_with_different_parents_is_applied() {
        let edges = Edges::new();
        edges.replace(NodeId(3), vec![slot(1)]);
        assert!(edges.replace(NodeId(3), vec![slot(2)]).is_some());
        assert!(!edges.is_watched(NodeId(1)));
        assert!(edges.is_watched(NodeId(2)));
    }

    #[test]
    fn unregister_clears_parent_from_child() {
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        edges.unregister(NodeId(1));
        assert_eq!(edges.count_parents_if(NodeId(2), |_| true), 0);
    }

    #[test]
    fn unregister_reports_unwatched() {
        let edges = Edges::new();
        edges.replace(NodeId(2), vec![slot(1)]);
        assert_eq!(edges.unregister(NodeId(2)), vec![NodeId(1)]);
    }
}
