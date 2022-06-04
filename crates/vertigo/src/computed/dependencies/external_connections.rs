use std::{
    rc::Rc,
};
use crate::{
    computed::graph_id::GraphId,
    struct_mut::BTreeMapMut, DropResource,
};

pub type ConnectType = Rc<dyn Fn() -> DropResource>;

struct ExternalConnectionsInner {
    connect: BTreeMapMut<GraphId, ConnectType>,
    connected_resource: BTreeMapMut<GraphId, DropResource>,
}

#[derive(Clone)]
pub struct ExternalConnections {
    inner: Rc<ExternalConnectionsInner>,
}

impl ExternalConnections {
    pub fn default() -> Self {
        ExternalConnections {
            inner: Rc::new(
                ExternalConnectionsInner {
                    connect: BTreeMapMut::new(),
                    connected_resource: BTreeMapMut::new(),
                }
            ),
        }
    }

    pub fn register_connect(&self, id: GraphId, connect: ConnectType) {
        self.inner.connect.insert(id, connect);
    }

    pub fn unregister_connect(&self, id: GraphId) {
        self.inner.connect.remove(&id);
    }

    pub fn set_connection(&self, id: GraphId, should_connect: bool) {
        if should_connect {
            if self.inner.connected_resource.contains_key(&id) {
                return;
            }

            //must be connected
            if let Some(connect_func) = self.inner.connect.get_and_clone(&id) {
                self.inner.connected_resource.insert(id, connect_func());
            }
        } else {
            self.inner.connected_resource.remove(&id);
        }
    }
}
