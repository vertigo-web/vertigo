use super::external_connections::ExternalConnections;
use super::graph_connections::GraphConnections;
use super::refresh::Refresh;
use crate::computed::graph_id::{GraphId, GraphIdKind};
use crate::Context;
use std::collections::BTreeSet;

pub struct Graph {
    pub(crate) refresh: Refresh,
    pub(crate) connections: GraphConnections,
    pub(crate) external_connections: ExternalConnections,
}

impl Graph {
    pub fn new() -> Graph {
        let external_connections = ExternalConnections::default();
        let refresh: Refresh = Refresh::new();

        Graph {
            refresh,
            connections: GraphConnections::new(),
            external_connections,
        }
    }

    pub(crate) fn remove_client(&self, client_id: GraphId) {
        self.set_parent_for_client(client_id, BTreeSet::new());
    }

    pub(crate) fn push_context(&self, client_id: GraphId, context: Context) {
        let parents = context.get_parents();
        self.set_parent_for_client(client_id, parents);
    }

    pub(crate) fn set_parent_for_client(
        &self,
        client_id: GraphId,
        parents_list: BTreeSet<GraphId>,
    ) {
        let edge_list = self
            .connections
            .set_parent_for_client(client_id, parents_list);

        for (id, active) in edge_list {
            match id.get_type() {
                GraphIdKind::Value => {
                    self.external_connections.set_connection(id, active);
                }
                GraphIdKind::Computed => {
                    if active {
                    } else {
                        self.refresh.clear_cache(&id);
                    }
                }
                GraphIdKind::Client => {
                    if active {
                    } else {
                        self.refresh.clear_cache(&id);
                    }
                }
            }
        }
    }
}
