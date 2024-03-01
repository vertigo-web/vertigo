use std::collections::BTreeSet;

use crate::{struct_mut::VecMut, GraphId};

pub struct Context {
    //In transaction
    parent_ids: VecMut<GraphId>,
}
impl Context {
    pub(crate) fn new() -> Context {
        Context {
            parent_ids: VecMut::new(),
        }
    }

    pub(crate) fn add_parent(&self, parent_id: GraphId) {
        self.parent_ids.push(parent_id);
    }

    pub(crate) fn get_parents(self) -> BTreeSet<GraphId> {
        let list = self.parent_ids.into_inner();
        list.into_iter().collect::<BTreeSet<_>>()
    }
}
