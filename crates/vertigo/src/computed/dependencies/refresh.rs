use std::collections::{BTreeMap};
use crate::computed::graph_id::GraphId;
use crate::computed::graph_value::GraphValueRefresh;

pub struct Refresh {
    refresh: BTreeMap<GraphId, GraphValueRefresh>,      //Reference to GraphValue for refreshing if necessary
}

impl Refresh {
    pub fn new() -> Refresh {
        Refresh {
            refresh: BTreeMap::new()
        }
    }

    pub fn refresh_token_add(&mut self, graph_value_refresh: GraphValueRefresh) {
        let id = graph_value_refresh.id;
        let prev_refresh = self.refresh.insert(id, graph_value_refresh);

        if prev_refresh.is_none() {
            //Correct transition
            return;
        }

        log::error!("Another refresh token has been overwritten");
    }

    pub fn refresh_token_drop(&mut self, id: GraphId) {
        self.refresh.remove(&id);
    }

    pub(crate) fn get(&self, id: &GraphId) -> Option<GraphValueRefresh> {
        if let Some(item) = self.refresh.get(id) {
            return Some(item.clone());
        }

        log::error!("Missing refresh token for(3) {:?}", id);
        None
    }
}