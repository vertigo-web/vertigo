use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::super::context::ParentList;
use super::{ErasedNode, NodeId};

/// Result of replacing a child's parents: nodes that gained or lost their last child.
pub(super) struct ParentDiff {
    pub became_watched: Vec<NodeId>,
    pub became_unwatched: Vec<NodeId>,
}

/// Bidirectional DAG plus strong parent refs (so parents outlive the child that lists them).
pub(super) struct Edges {
    child_parents: RefCell<HashMap<NodeId, HashSet<NodeId>>>,
    parent_children: RefCell<HashMap<NodeId, HashSet<NodeId>>>,
    parent_refs: RefCell<HashMap<NodeId, Vec<Rc<dyn ErasedNode>>>>,
}

impl Edges {
    pub(super) fn new() -> Self {
        Self {
            child_parents: RefCell::new(HashMap::new()),
            parent_children: RefCell::new(HashMap::new()),
            parent_refs: RefCell::new(HashMap::new()),
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

    /// Replace `child`'s parent set. `None` means the parent ids were already the same.
    pub(super) fn replace(&self, child: NodeId, pairs: ParentList) -> Option<ParentDiff> {
        {
            let child_parents = self.child_parents.borrow();
            if let Some(old) = child_parents.get(&child)
                && Self::same_parent_ids(old, &pairs)
            {
                return None;
            }
        }

        let new_parents: HashSet<NodeId> = pairs.iter().map(|(id, _)| *id).collect();
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

    fn same_parent_ids(old: &HashSet<NodeId>, pairs: &ParentList) -> bool {
        if old.is_empty() {
            return pairs.is_empty();
        }
        for (id, _) in pairs {
            if !old.contains(id) {
                return false;
            }
        }
        if pairs.len() < old.len() {
            return false;
        }
        old.iter()
            .all(|id| pairs.iter().any(|(parent, _)| parent == id))
    }
}
