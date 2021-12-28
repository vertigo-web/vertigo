use std::collections::BTreeSet;
use std::rc::Rc;
use crate::computed::graph_id::GraphId;
use crate::struct_mut::BTreeMapMut;
use super::external_connections::ExternalConnections;
use super::graph_map::GraphMap;

pub struct Graph {
    parent_childs: GraphMap,                    // ParentId <- ClientId
    counters: BTreeMapMut<(GraphId, GraphId), u8>, // Relation counter
    external_connections: ExternalConnections,
}

impl Graph {
    pub fn new(external_connections: ExternalConnections) -> Graph {
        Graph {
            parent_childs: GraphMap::new(),
            counters: BTreeMapMut::new(),
            external_connections,
        }
    }

    pub fn add_graph_connection(&self, parent_id_set: Rc<BTreeSet<GraphId>>, client_id: GraphId) {
        for parent_id in parent_id_set.iter() {
            let id = (*parent_id, client_id);
            let increase_success = self.counters.get_mut(&id, |counter| {
                *counter += 1;
            });

            if increase_success.is_none() {
                 
                self.parent_childs.add_connection(*parent_id, client_id);
                self.counters.insert(id, 1);

                //Connect start
                self.external_connections.need_connection(*parent_id);
            }
        }
    }

    pub fn remove_graph_connection(&self, parent_id_set: &Rc<BTreeSet<GraphId>>, client_id: GraphId) {
        for parent_id in parent_id_set.iter() {
            let id = (*parent_id, client_id);
            let should_clear = self.counters.get_mut(&id, |counter| {
                if *counter > 0 {
                    *counter -= 1;

                    *counter == 0
                } else {
                    log::error!("More than zero was expected");
                    true
                }
            });

            if let Some(should_clear) = should_clear {
                if should_clear {
                    self.parent_childs.remove_connection(*parent_id, client_id);
                    self.counters.remove(&id);

                    //Connect down
                    self.external_connections.need_disconnection(*parent_id);
                }
            } else {
                log::error!("Counters missing");
            }
        }
    }

    pub(crate) fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        self.parent_childs.get_all_deps(edges)
    }

    pub fn has_listeners(&self, parent_id: &GraphId) -> bool {
        self.parent_childs.relation_len(parent_id) > 0
    }

    pub fn all_connections_len(&self) -> u64 {
        self.counters.map(|counters| {
            let mut count: u64 = 0;

            for item_count in counters.values() {
                count += *item_count as u64;
            }

            count
        })
    }

    pub fn all_connections(&self) -> Vec<(GraphId, GraphId, u8)> {
        self.counters.map(|counters| {
            let mut result = Vec::new();

            for ((parent_id, client_id), counter) in counters {
                result.push((*parent_id, *client_id, *counter))
            }

            result
        })
    }
}
