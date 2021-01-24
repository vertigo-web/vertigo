use std::collections::{BTreeMap, BTreeSet};
use crate::computed::graph_id::GraphId;
use super::graph_map::GraphMap;

pub struct Graph {
    parent_childs: GraphMap,                            //ParentId <- ClientId
    client_parents: GraphMap,                           //ClientId <- ParentId
    counters: BTreeMap<(GraphId, GraphId), u8>,         //Relation counter
    will_be_dropped: BTreeSet<GraphId>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            parent_childs: GraphMap::new(),
            client_parents: GraphMap::new(),
            counters: BTreeMap::new(),
            will_be_dropped: BTreeSet::new(),
        }
    }

    pub fn add_graph_connection(&mut self, parent_id: GraphId, client_id: GraphId) {
        self.will_be_dropped.remove(&parent_id);

        let id = (parent_id, client_id);
        let counter = self.counters.get_mut(&id);

        if let Some(counter) = counter {
            *counter += 1;
            return;
        }

        self.parent_childs.add_connection(parent_id, client_id);
        self.client_parents.add_connection(client_id, parent_id);
        self.counters.insert(id, 1);
    }

    pub fn remove_graph_connection(&mut self, parent_id: GraphId, client_id: GraphId) {
        let id = (parent_id, client_id);
        let counter = self.counters.get_mut(&id);

        let should_clear = if let Some(counter) = counter {
            if *counter > 0 {
                *counter -= 1;

                *counter == 0
            } else {
                log::error!("More than zero was expected");
                return;
            }
        } else {
            log::error!("Counters missing");
            return;
        };

        if should_clear {
            self.parent_childs.remove_connection(parent_id, client_id);
            self.client_parents.remove_connection(client_id, parent_id);
            self.counters.remove(&id);

            if self.parent_childs.relation_len(&parent_id) == 0 {
                self.will_be_dropped.insert(parent_id);
            }

            /*
            if self.client_parents.relation_len(&client_id) == 0 {
                let graph_value = self.refresh.get(&client_id);
                if let Some(graph_value) = graph_value {
                    graph_value.drop_value();
                } else {
                    log::error!("Refresh token missing");
                }
            }
            */
        }
    }

    pub(crate) fn get_all_deps(&self, edges: &BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        let mut result = BTreeSet::new();
        let mut to_traverse: Vec<GraphId> = Vec::new();

        for item in edges.iter() {
            to_traverse.push(*item);
        }

        loop {
            let next_to_traverse = to_traverse.pop();

            match next_to_traverse {
                Some(next) => {
                    let list = self.parent_childs.get_relation(&next);

                    if let Some(list) = list {
                        for item in list {
                            let is_contain = result.contains(item);
                            if !is_contain {
                                result.insert(*item);
                                to_traverse.push(*item);
                            }
                        }
                    }
                },
                None => {
                    return result;
                }
            }
        }
    }

    pub fn has_listeners(&self, parent_id: &GraphId) -> bool {
        self.parent_childs.relation_len(&parent_id) > 0
    }

    pub fn drain_removables(&mut self) -> Vec<GraphId> {
        let mut result = Vec::new();

        for item in &self.will_be_dropped {
            result.push(*item);
        }

        self.will_be_dropped = BTreeSet::new();

        result
    }
    

    pub fn get_parents(&self, client_id: GraphId) -> Vec<GraphId> {
        if let Some(item) = self.client_parents.get_relation(&client_id) {
            let mut result: Vec<GraphId> = Vec::new();

            for parent_id in item.iter() {
                result.push(*parent_id);
            }

            return result;
        }

        Vec::new()
    }

    pub fn all_connections_len(&self) -> u64 {
        let mut count: u64 = 0;

        for item_count in self.counters.values() { //}: BTreeMap<(GraphId, GraphId), u8>,
            count += *item_count as u64;
        }

        count
    }

    pub fn all_connections(&self) -> Vec<(GraphId, GraphId, u8)> {
        let mut result = Vec::new();

        for ((parent_id, client_id), counter) in &self.counters {
            result.push((*parent_id, *client_id, *counter))
        }

        result
    }
}
