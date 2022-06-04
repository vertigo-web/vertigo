use std::rc::Rc;
use crate::{computed::{graph_id::GraphId, graph_value::GraphValueRefresh}, struct_mut::BTreeMapMut};

#[derive(Clone)]
pub struct Refresh {
    refresh: Rc<BTreeMapMut<GraphId, GraphValueRefresh>>, // Reference to GraphValue for refreshing if necessary
}

impl Refresh {
    pub fn new() -> Refresh {
        Refresh {
            refresh: Rc::new(BTreeMapMut::new()),
        }
    }

    pub fn refresh_token_add(&self, graph_value_refresh: GraphValueRefresh) {
        let id = graph_value_refresh.id();
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

    fn get(&self, id: &GraphId) -> Option<GraphValueRefresh> {
        if let Some(item) = self.refresh.get_and_clone(id) {
            return Some(item);
        }

        None
    }

    pub(crate) fn refresh(&self, id: &GraphId) {
        if let Some(item) = self.get(id) {
            item.refresh();
        } else {
        }
    }

    pub(crate) fn clear_cache(&self, id: &GraphId) {
        if let Some(item) = self.get(id) {
            item.clear_cache();
        } else {
        }
    }
}
