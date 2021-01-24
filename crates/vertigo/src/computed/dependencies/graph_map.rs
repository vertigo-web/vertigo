use std::collections::{BTreeMap, BTreeSet};
use crate::computed::graph_id::GraphId;

pub struct GraphMap {
    data: BTreeMap<GraphId, BTreeSet<GraphId>>      //A <- B
}

impl GraphMap {
    pub fn new() -> GraphMap {
        GraphMap {
            data: BTreeMap::new(),
        }
    }

    pub fn add_connection(&mut self, parent_id: GraphId, client_id: GraphId) {
        self.data
            .entry(parent_id)
            .or_insert_with(BTreeSet::new)
            .insert(client_id);
    }

    pub fn remove_connection(&mut self, parent_id: GraphId, client_id: GraphId) {
        let parent_list = self.data.get_mut(&parent_id);

        let should_clear = if let Some(parent_list) = parent_list {
            parent_list.remove(&client_id);
            parent_list.is_empty()
        } else {
            log::error!("Missing relation in GraphMap");
            false
        };

        if should_clear {
            self.data.remove(&parent_id);
        }
    }

    pub fn relation_len(&self, id: &GraphId) -> usize {
        let item = self.data.get(id);

        if let Some(item) = item {
            item.len()
        } else {
            0
        }
    }

    pub fn get_relation(&self, id: &GraphId) -> Option<&BTreeSet<GraphId>> {
        self.data.get(id)
    }
}
