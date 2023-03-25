use std::rc::Rc;

use crate::{computed::graph_id::GraphId, struct_mut::BTreeMapMut};

pub struct Refresh {
    refresh: BTreeMapMut<GraphId, Rc<dyn Fn(bool) + 'static>>, // Reference to GraphValue for refreshing if necessary
}

impl Refresh {
    pub fn new() -> Refresh {
        Refresh {
            refresh: BTreeMapMut::new(),
        }
    }

    pub fn refresh_token_add(&self, id: GraphId, graph_value_refresh: impl Fn(bool) + 'static) {
        let prev_refresh = self.refresh.insert(id, Rc::new(graph_value_refresh));

        if prev_refresh.is_none() {
            //Correct transition
            return;
        }

        log::error!("Another refresh token has been overwritten");
    }

    pub fn refresh_token_drop(&self, id: GraphId) {
        self.refresh.remove(&id);
    }

    fn get(&self, id: &GraphId) -> Option<Rc<dyn Fn(bool)>> {
        if let Some(item) = self.refresh.get_and_clone(id) {
            return Some(item);
        }

        None
    }

    pub(crate) fn refresh(&self, id: &GraphId) {
        if let Some(refresh) = self.get(id) {
            refresh(true);
        }
    }

    pub(crate) fn clear_cache(&self, id: &GraphId) {
        if let Some(refresh) = self.get(id) {
            refresh(false);
        }
    }
}
