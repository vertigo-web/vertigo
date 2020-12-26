use std::collections::{BTreeMap, BTreeSet};
use crate::computed::graph_id::GraphId;
use crate::computed::graph_value::GraphValueRefresh;
use super::graph_map::GraphMap;

pub struct Graph {
    parent_childs: GraphMap,                            //ParentId <- ClientId
    client_parents: GraphMap,                           //ClientId <- ParentId
    refresh: BTreeMap<GraphId, GraphValueRefresh>,      //Reference to GraphValue for refreshing if necessary
    counters: BTreeMap<(GraphId, GraphId), u8>,         //Relation counter
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            parent_childs: GraphMap::new(),
            client_parents: GraphMap::new(),
            refresh: BTreeMap::new(),
            counters: BTreeMap::new(),
        }
    }

    pub fn add_graph_connection(&mut self, parent_id: GraphId, client_id: GraphId) {
        let id = (parent_id, client_id);
        let counter = self.counters.get_mut(&id);

        if let Some(counter) = counter {
            *counter += 1;
            return;
        }

        self.parent_childs.add_connection(parent_id.clone(), client_id.clone());
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
            self.parent_childs.remove_connection(parent_id.clone(), client_id.clone());
            self.client_parents.remove_connection(client_id, parent_id);
            self.counters.remove(&id);

            if self.client_parents.relation_len(&client_id) == 0 {
                let graph_value = self.refresh.remove(&client_id);
                if let Some(graph_value) = graph_value {
                    graph_value.drop_value();
                } else {
                    log::error!("Refresh token missing");
                }
            }
        }
    }

    pub fn report_graph_value_as_refresh_token(&mut self, graph_value_refresh: GraphValueRefresh) {
        let id = graph_value_refresh.id;
        let prev_refresh = self.refresh.insert(id, graph_value_refresh);

        if prev_refresh.is_none() {
            //Correct transition
            return;
        }

        log::error!("Another refresh token has been overwritten");
    }

    fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        let mut result = BTreeSet::new();
        let mut to_traverse: Vec<GraphId> = edges.into_iter().collect();

        loop {
            let next_to_traverse = to_traverse.pop();

            match next_to_traverse {
                Some(next) => {
                    let list = self.parent_childs.get_relation(&next);

                    if let Some(list) = list {
                        for item in list {
                            let is_contain = result.contains(item);
                            if !is_contain {
                                result.insert(item.clone());
                                to_traverse.push(item.clone());
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

    pub fn get_edges_to_refresh(&self, edges: BTreeSet<GraphId>) -> Vec<GraphValueRefresh> {

        let mut result = Vec::new();

        for id in self.get_all_deps(edges) {
            if let Some(item) = self.refresh.get(&id) {
                result.push((*item).clone());
            } else {
                log::error!("Missing refresh token for {:?}", id);
            }
        }

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
}
