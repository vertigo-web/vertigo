use std::{cell::RefCell, rc::Rc};

use super::graph::{ErasedNode, NodeId};

pub(crate) type ParentList = Vec<(NodeId, Rc<dyn ErasedNode>)>;

/// Tracking context passed into [`crate::Value::get`] / [`crate::Computed::get`].
pub struct Context {
    pub(crate) parents: Option<RefCell<ParentList>>,
}

impl Context {
    pub(crate) fn read() -> Self {
        Context { parents: None }
    }

    pub(crate) fn tracking() -> Self {
        Context {
            parents: Some(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn track(&self, id: NodeId, slot: Rc<dyn ErasedNode>) {
        if let Some(parents) = &self.parents {
            parents.borrow_mut().push((id, slot));
        }
    }

    pub(crate) fn take_parents(&self) -> ParentList {
        match &self.parents {
            Some(parents) => parents.take(),
            None => Vec::new(),
        }
    }
}
