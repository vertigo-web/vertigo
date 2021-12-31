use std::{
    any::Any,
    rc::Rc,
};

use crate::{
    computed::graph_id::GraphId,
    utils::{EqBox}, struct_mut::BTreeMapMut,
};

pub type ConnectType = Rc<dyn Fn() -> Box<dyn Any>>;

struct ExternalConnectionsInner {
    connect: BTreeMapMut<GraphId, ConnectType>,
    connect_resource: BTreeMapMut<GraphId, Box<dyn Any>>,
    will_connect: BTreeMapMut<GraphId, bool>,
}

#[derive(PartialEq, Clone)]
pub struct ExternalConnections {
    inner: Rc<EqBox<ExternalConnectionsInner>>,
}

impl ExternalConnections {
    pub fn default() -> Self {
        ExternalConnections {
            inner: Rc::new(EqBox::new(
                ExternalConnectionsInner {
                    connect: BTreeMapMut::new(),
                    connect_resource: BTreeMapMut::new(),
                    will_connect: BTreeMapMut::new(),
                }
            )),
        }
    }

    pub fn register_connect(&self, id: GraphId, connect: ConnectType) {
        self.inner.connect.insert(id, connect);
    }

    pub fn unregister_connect(&self, id: GraphId) {
        self.inner.connect.remove(&id);
    }

    pub fn need_connection(&self, id: GraphId) {
        self.inner.will_connect.insert(id, true);
    }

    pub fn need_disconnection(&self, id: GraphId) {
        self.inner.will_connect.insert(id, false);
    }

    pub fn refresh_connect(&self) {
        let will_connect = self.inner.will_connect.take();
        for (id, should_connect) in will_connect.into_iter() {
            if should_connect {
                if !self.inner.connect_resource.contains_key(&id) {
                    //must be connected
                    if let Some(connect_func) = self.inner.connect.get(&id) {
                        let connect_resource = connect_func();
                        self.inner.connect_resource.insert(id, connect_resource);
                    }
                }
            } else {
                self.inner.connect_resource.remove(&id);
            }
        }
    }
}
