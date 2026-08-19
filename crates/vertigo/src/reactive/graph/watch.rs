use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::DropResource;
use super::{NodeId, edges::ParentDiff};

/// `when_connect` lifecycle: run `connect` while a node has children, drop the resource when not.
pub(super) struct Watch {
    connect: RefCell<HashMap<NodeId, Rc<dyn Fn() -> DropResource>>>,
    connected: RefCell<HashMap<NodeId, DropResource>>,
}

impl Watch {
    pub(super) fn new() -> Self {
        Self {
            connect: RefCell::new(HashMap::new()),
            connected: RefCell::new(HashMap::new()),
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
            self.on_watched(id);
        }
    }

    pub(super) fn unregister(&self, id: NodeId) {
        self.connect.borrow_mut().remove(&id);
        let _dropped = self.connected.borrow_mut().remove(&id);
    }

    pub(super) fn apply(&self, diff: ParentDiff) {
        for id in diff.became_watched {
            self.on_watched(id);
        }
        for id in diff.became_unwatched {
            self.on_unwatched(id);
        }
    }

    pub(super) fn on_unwatched_many(&self, ids: Vec<NodeId>) {
        for id in ids {
            self.on_unwatched(id);
        }
    }

    fn on_watched(&self, id: NodeId) {
        if self.connected.borrow().contains_key(&id) {
            return;
        }
        let Some(connect) = self.connect.borrow().get(&id).cloned() else {
            return;
        };
        let resource = connect();
        self.connected.borrow_mut().insert(id, resource);
    }

    fn on_unwatched(&self, id: NodeId) {
        let _dropped = self.connected.borrow_mut().remove(&id);
    }
}
