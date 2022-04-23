use std::collections::BTreeSet;
use crate::computed::graph_id::GraphId;
use super::external_connections::ExternalConnections;
use super::graph_map::GraphMap;
use super::refresh::Refresh;

pub struct Graph {
    refresh: Refresh,
    parent_client: GraphMap,                    // ParentId <- ClientId
    client_parent: GraphMap,
    external_connections: ExternalConnections,
}

impl Graph {
    pub fn new(external_connections: ExternalConnections, refresh: Refresh) -> Graph {
        Graph {
            refresh,
            parent_client: GraphMap::new(),
            client_parent: GraphMap::new(),
            external_connections,
        }
    }

    pub fn set_parent_for_client(&self, client_id: GraphId, parents_list: BTreeSet<GraphId>) {        
        let (off, on) = self.client_parent.set_connectios(&client_id, parents_list);

        self.edges_on(client_id, on);
        self.edges_off(client_id, off);
    }

    pub fn remove_client(&self, client_id: GraphId) {
        let parent_list = self.client_parent.remove_by(client_id);

        let parent_list = match parent_list {
            Some(parent) => parent,
            None => {
                return;
            }
        };

        self.edges_off(client_id, parent_list);
    }

    fn edges_on(&self, client_id: GraphId, parent_list: BTreeSet<GraphId>) {
        self.parent_client.add_connection(&parent_list, client_id);
        self.external_connections.need_connection(parent_list);
    }

    fn edges_off(&self, client_id: GraphId, parent_list: BTreeSet<GraphId>) {
        for parent_id in parent_list {
            self.parent_client.remove_connection(parent_id, client_id);
            self.external_connections.need_disconnection(parent_id);

            if self.parent_client.relation_len(&parent_id) == 0 {
                if let Some(token) = self.refresh.get(&parent_id) {
                    token.drop_value();
                }
            }
        }
    }

    pub(crate) fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        self.parent_client.get_all_deps(edges)
    }

    pub fn all_connections_len(&self) -> u64 {
        self.parent_client.relation_len_all()
    }
}
