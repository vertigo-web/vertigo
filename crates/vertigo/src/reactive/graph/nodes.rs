use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::{ErasedNode, NodeId, node_hash::NodeMap};

/// Weak registry of live nodes.
pub(super) struct Nodes {
    slots: RefCell<NodeMap<Weak<dyn ErasedNode>>>,
}

impl Nodes {
    pub(super) fn new() -> Self {
        Self {
            slots: RefCell::new(NodeMap::default()),
        }
    }

    pub(super) fn register(&self, id: NodeId, slot: Rc<dyn ErasedNode>) {
        self.slots.borrow_mut().insert(id, Rc::downgrade(&slot));
    }

    pub(super) fn upgrade(&self, id: NodeId) -> Option<Rc<dyn ErasedNode>> {
        self.slots.borrow().get(&id).and_then(Weak::upgrade)
    }

    pub(super) fn remove(&self, id: NodeId) {
        self.slots.borrow_mut().remove(&id);
    }
}
