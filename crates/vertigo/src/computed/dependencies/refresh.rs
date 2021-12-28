use crate::{computed::{graph_id::GraphId, graph_value::GraphValueRefresh}, struct_mut::BTreeMapMut};

pub struct Refresh {
    refresh: BTreeMapMut<GraphId, GraphValueRefresh>, // Reference to GraphValue for refreshing if necessary
}

impl Refresh {
    pub fn new() -> Refresh {
        Refresh {
            refresh: BTreeMapMut::new(),
        }
    }

    pub fn refresh_token_add(&self, graph_value_refresh: GraphValueRefresh) {
        let id = graph_value_refresh.id;
        let prev_refresh = self.refresh.insert(id, graph_value_refresh);

        if prev_refresh.is_none() {
            //Correct transition
            return;
        }

        log::error!("Another refresh token has been overwritten");
    }

    pub fn refresh_token_drop(&self, id: GraphId) {
        self.refresh.remove(&id);
    }

    pub(crate) fn get(&self, id: &GraphId) -> Option<GraphValueRefresh> {
        if let Some(item) = self.refresh.get(id) {
            return Some(item);
        }

        log::error!("Missing refresh token for(3) {:?}", id);
        None
    }
}
