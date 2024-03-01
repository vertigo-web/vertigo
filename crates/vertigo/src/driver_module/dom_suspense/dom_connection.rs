use crate::DomId;
use std::collections::{BTreeSet, HashMap};

pub struct DomConnection {
    parent: HashMap<DomId, DomId>,
    child: HashMap<DomId, BTreeSet<DomId>>, //parent -> child (BTreeSet<GraphId>)
}

impl DomConnection {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            child: HashMap::new(),
        }
    }

    pub fn set_parent(&mut self, node_id: DomId, parent_id: DomId) {
        self.node_remove(node_id);
        self.node_insert(node_id, parent_id);
    }

    pub fn remove(&mut self, node_id: DomId) {
        self.node_remove(node_id);
    }

    fn node_remove(&mut self, node_id: DomId) {
        let parent_id = self.parent.remove(&node_id);

        if let Some(parent_id) = parent_id {
            self.child.remove(&parent_id);
        }
    }

    fn node_insert(&mut self, node_id: DomId, parent_id: DomId) {
        self.parent.insert(node_id, parent_id);

        self.child.entry(parent_id).or_default().insert(node_id);
    }

    pub fn get_parent(&self, node_id: DomId) -> Option<DomId> {
        self.parent.get(&node_id).cloned()
    }
}
