use std::{
    any::Any,
    collections::BTreeMap,
    rc::Rc,
};

use crate::{
    computed::graph_id::GraphId,
    utils::{BoxRefCell, EqBox},
};

pub type ConnectType = Box<dyn Fn() -> Box<dyn Any>>;

struct ExternalConnectionsInner {
    connect: BTreeMap<GraphId, ConnectType>,
    connect_resource: BTreeMap<GraphId, Box<dyn Any>>,
    will_connect: BTreeMap<GraphId, bool>,
}

impl ExternalConnectionsInner {
    fn new() -> ExternalConnectionsInner {
        ExternalConnectionsInner {
            connect: BTreeMap::new(),
            connect_resource: BTreeMap::new(),
            will_connect: BTreeMap::new(),
        }
    }

    fn register_connect(&mut self, id: GraphId, connect: ConnectType) {
        self.connect.insert(id, connect);
    }

    fn unregister_connect(&mut self, id: GraphId) {
        self.connect.remove(&id);
    }

    fn connect(&mut self, id: GraphId) {
        if self.connect_resource.contains_key(&id) {
            return;
        }

        //must be connected

        if let Some(connect_func) = self.connect.get(&id) {
            let connect_resource = connect_func();
            self.connect_resource.insert(id, connect_resource);
        }
    }

    fn disconnect(&mut self, id: GraphId) {
        self.connect_resource.remove(&id);
    }

    fn refresh_connect(&mut self) {
        if self.will_connect.is_empty() {
            return;
        }

        let will_connect = std::mem::take(&mut self.will_connect);

        for (id, should_connect) in will_connect.into_iter() {
            if should_connect {
                self.connect(id);
            } else {
                self.disconnect(id);
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct ExternalConnections {
    inner: Rc<EqBox<BoxRefCell<ExternalConnectionsInner>>>,
}

impl ExternalConnections {
    pub fn default() -> Self {
        ExternalConnections {
            inner: Rc::new(EqBox::new(BoxRefCell::new(
                ExternalConnectionsInner::new(),
                "ExternalConnections",
            ))),
        }
    }

    pub fn register_connect(&self, id: GraphId, connect: ConnectType) {
        self.inner.change((id, connect), |state, (id, connect)| {
            state.register_connect(id, connect);
        });
    }

    pub fn unregister_connect(&self, id: GraphId) {
        self.inner.change(id, |state, id| {
            state.unregister_connect(id);
        });
    }

    pub fn need_connection(&self, id: GraphId) {
        self.inner.change(id, |state, id| {
            state.will_connect.insert(id, true);
        });
    }

    pub fn need_disconnection(&self, id: GraphId) {
        self.inner.change(id, |state, id| {
            state.will_connect.insert(id, false);
        });
    }

    pub fn refresh_connect(&self) {
        self.inner.change((), |state, _| {
            state.refresh_connect();
        });
    }
}
