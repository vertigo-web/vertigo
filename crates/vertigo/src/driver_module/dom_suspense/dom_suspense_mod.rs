use std::{collections::BTreeMap, rc::Rc};

use crate::{struct_mut::InnerValue, DomId, DropResource};

use super::{dom_connection::DomConnection, NodePaths};

struct LayerCallback {
    callback: Rc<dyn Fn(bool)>,
    last_show_loading: bool,
}

struct DomSuspenseState {
    dom_connection: DomConnection,
    suspense: NodePaths,                //<vertigo-suspense />, node ID -> the node and all its ancestors
    layer: NodePaths,                   //layer ID -> full path
    layer_callback: BTreeMap<DomId, LayerCallback>,
}

impl DomSuspenseState {
    pub fn new() -> Self {
        Self {
            dom_connection: DomConnection::new(),
            suspense: NodePaths::new(),
            layer: NodePaths::new(),
            layer_callback: BTreeMap::new(),
        }
    }

    fn refresh_subscribers(&mut self) {
        for (id, layer) in self.layer_callback.iter_mut() {
            if let Some(id) = self.dom_connection.get_parent(*id) {
                let show_loading = self.suspense.contains(&id);

                if layer.last_show_loading != show_loading {
                    layer.last_show_loading = show_loading;
                    (layer.callback)(show_loading);
                }
            }
        }
    }

    fn refresh_paths(&mut self) {
        self.suspense.refresh_paths(&self.dom_connection);
        self.layer.refresh_paths(&self.dom_connection);

        self.refresh_subscribers();
    }

    pub fn set_node_suspense(&mut self, suspense_id: DomId) {
        self.suspense.insert(&self.dom_connection, suspense_id);
        self.refresh_paths();
    }

    pub fn set_parent(&mut self, node_id: DomId, parent_id: DomId) {
        self.dom_connection.set_parent(node_id, parent_id);

        self.refresh_paths();
    }

    pub fn remove(&mut self, node_id: DomId) {
        self.dom_connection.remove(node_id);
        let is_suspense_removed = self.suspense.remove(node_id);
        let is_layer_removed = self.layer.remove(node_id);

        if is_suspense_removed || is_layer_removed {
            self.refresh_paths();
        }
    }

    pub fn set_layer_callback(&mut self, node_id: DomId, callback: impl Fn(bool) + 'static) {
        let show_loading = self.suspense.contains(&node_id);

        let callback = Rc::new(callback);
        callback(show_loading);

        let state = LayerCallback {
            callback,
            last_show_loading: show_loading,
        };

        let prev_value = self.layer_callback.insert(node_id, state);

        if prev_value.is_some() {
            log::error!("set_layer_callback: Previous subscription has been overwritten");
        }

        self.layer.insert(&self.dom_connection, node_id);
        self.refresh_paths();
    }

    pub fn remove_layer_callback(&mut self, node_id: DomId) {
        self.layer_callback.remove(&node_id);
        let is_layer_removed = self.layer.remove(node_id);

        if is_layer_removed {
            self.refresh_paths();
        }
    }
}

pub struct DomSuspense {
    inner: InnerValue<DomSuspenseState>,
}

impl DomSuspense {
    pub fn new() -> &'static DomSuspense {
        Box::leak(Box::new(Self {
            inner: InnerValue::new(DomSuspenseState::new())
        }))
    }

    //Function called when a new <vertigo-suspense /> node is created
    pub fn set_node_suspense(&self, node_id: DomId) {
        self.inner.get_mut().set_node_suspense(node_id);
    }

    //Called when a node is attached to a parent
    pub fn set_parent(&self, node_id: DomId, parent_id: DomId) {
        self.inner.get_mut().set_parent(node_id, parent_id);
    }

    //Called when a node is destroyed
    pub fn remove(&self, node_id: DomId) {
        self.inner.get_mut().remove(node_id);
    }

    //Set up a subscription that provides information about whether a layer should be shown or hidden
    //This function will be used in DomElement when the vertigo-suspense attribute is set
    pub fn set_layer_callback(&'static self, node_id: DomId, callback: impl Fn(bool) + 'static) -> DropResource {
        self.inner.get_mut().set_layer_callback(node_id, callback);

        DropResource::new(move || {
            self.inner.get_mut().remove_layer_callback(node_id);
        })
    }
}
