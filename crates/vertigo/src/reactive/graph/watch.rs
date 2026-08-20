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
        assert_eq!(connects.get(), 1);
    }

    #[test]
    fn unregister_drops_connected() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.unregister(NodeId(1));
        assert_eq!(live.get(), 0);
    }

    #[test]
    fn apply_unwatched_drops_connected() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.apply(ParentDiff {
            became_watched: Vec::new(),
            became_unwatched: vec![NodeId(1)],
        });
        assert_eq!(live.get(), 0);
    }

    #[test]
    fn unwatched_many_drops_connected() {
        let watch = Watch::new();
        let live = Rc::new(Cell::new(0));
        watch.register(NodeId(1), connect_flag(&live), true);
        watch.on_unwatched_many(vec![NodeId(1)]);
        assert_eq!(live.get(), 0);
    }
}
