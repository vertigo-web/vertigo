use std::collections::{BTreeSet};

use crate::{computed::graph_id::GraphId, struct_mut::BTreeMapMut};

pub struct GraphMap {
    data: BTreeMapMut<GraphId, BTreeSet<GraphId>>, // A <- B
}

impl GraphMap {
    pub fn new() -> GraphMap {
        GraphMap {
            data: BTreeMapMut::new(),
        }
    }

    pub fn add_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.data.change(|data| {
            data
                .entry(parent_id)
                .or_insert_with(BTreeSet::new)
                .insert(client_id);
        })
    }

    pub fn remove_connection(&self, parent_id: GraphId, client_id: GraphId) {
        let should_clear = self.data.get_mut(&parent_id, |parent_list| {
            parent_list.remove(&client_id);
            parent_list.is_empty()
        });

        match should_clear {
            Some(true) => {
                self.data.remove(&parent_id);
            },
            Some(false) => {},
            None => {
                log::error!("Missing relation in GraphMap");
            }
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

    pub(crate) fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        self.data.map(move |state| -> BTreeSet<GraphId> {
            let mut result = BTreeSet::new();
            let mut to_traverse: Vec<GraphId> = edges.into_iter().collect();

            loop {
                let next_to_traverse = to_traverse.pop();

                match next_to_traverse {
                    Some(next) => {
                        let list = state.get(&next);

                        if let Some(list) = list {
                            for item in list {
                                let is_contain = result.contains(item);
                                if !is_contain {
                                    result.insert(*item);
                                    to_traverse.push(*item);
                                }
                            }
                        }
                    }
                    None => {
                        return result;
                    }
                }
            }
        })
    }
}
