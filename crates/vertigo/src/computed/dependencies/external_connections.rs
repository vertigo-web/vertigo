use std::rc::Rc;

use crate::computed::{DropResource, GraphId, struct_mut::BTreeMapMut};

pub type ConnectType = Rc<dyn Fn() -> DropResource>;

pub struct ExternalConnections {
    connect: BTreeMapMut<GraphId, ConnectType>,
    connected_resource: BTreeMapMut<GraphId, DropResource>,
}

impl ExternalConnections {
    pub fn default() -> Self {
        ExternalConnections {
            connect: BTreeMapMut::new(),
            connected_resource: BTreeMapMut::new(),
        }
    }

    pub fn register_connect(&self, id: GraphId, connect: ConnectType) {
        self.connect.insert(id, connect);
    }

    pub fn unregister_connect(&self, id: GraphId) {
        self.connect.remove(&id);
    }

    pub fn set_connection(&self, id: GraphId, should_connect: bool) {
        if should_connect {
            if self.connected_resource.contains_key(&id) {
                return;
            }

            //must be connected
            if let Some(connect_func) = self.connect.get_and_clone(&id) {
                self.connected_resource.insert(id, connect_func());
            }
        } else {
            self.connected_resource.remove(&id);
        }
    }
}
