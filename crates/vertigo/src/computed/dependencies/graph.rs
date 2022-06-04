use std::collections::BTreeSet;
use crate::computed::dependencies::stack::StackCommand;
use crate::computed::graph_id::{GraphId, GraphIdKind};
use crate::struct_mut::ValueMut;
use super::external_connections::ExternalConnections;
use super::graph_connections::GraphConnections;
use super::refresh::Refresh;
use super::stack::Stack;

pub struct Graph {
    recalculate_in_progress: ValueMut<bool>,
    pub stack: Stack,
    pub refresh: Refresh,
    pub connections: GraphConnections,
    pub external_connections: ExternalConnections,
}

impl Graph {
    pub fn new() -> Graph {
        let external_connections = ExternalConnections::default();
        let refresh: Refresh = Refresh::new();

        Graph {
            recalculate_in_progress: ValueMut::new(false),
            stack: Stack::new(),
            refresh,
            connections: GraphConnections::new(),
            external_connections,
        }
    }

    ///used by Drop in GraphValue
    pub fn remove_client(&self, client_id: GraphId) {
        self.stack.remove_client(client_id);
        self.recalculate_edges();
    }

    pub(crate) fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        self.connections.get_all_deps(edges)
    }

    pub fn all_connections_len(&self) -> u64 {
        self.connections.all_connections_len()
    }

    ///This method can only be executed once
    pub fn recalculate_edges(&self) {
        if !self.stack.is_empty() {
            return;
        }

        if self.recalculate_in_progress.get() {
            return;
        }

        self.recalculate_in_progress.set(true);

        let mut off_edges_all = BTreeSet::new();

        loop {
            let command = self.stack.get_command();

            if command.is_empty() {
                break;
            }

            let mut local_off_edges = BTreeSet::new();

            for action in command.into_iter() {
                let off_edges = match action {
                    StackCommand::Set { parent_ids, client_id } => {
                        off_edges_all.insert(client_id);
                        local_off_edges.extend(parent_ids.iter());
                        self.connections.set_parent_for_client(parent_ids, client_id)
                    },
                    StackCommand::Remove { client_id } => {
                        self.connections.set_parent_for_client(BTreeSet::new(), client_id)
                    }
                };

                if let Some(off_edges) = off_edges {
                    local_off_edges.extend(off_edges.iter());
                }
            }

            for id in local_off_edges.iter() {
                self.should_clear_node(id);
            }

            off_edges_all.extend(local_off_edges.into_iter());
        }

        for id in off_edges_all {
            self.should_clear_node(&id);

            if let GraphIdKind::Value = id.get_type() {
                let has_subscribers = self.connections.has_subscribers(&id);
                self.external_connections.set_connection(id, has_subscribers);
            }
        }

        self.recalculate_in_progress.set(false);

    }

    fn should_clear_node(&self, id: &GraphId) {
        match id.get_type() {
            GraphIdKind::Value => {
            },
            GraphIdKind::Computed => {
                let has_subscribers = self.connections.has_subscribers(id);
    
                if !has_subscribers {
                    self.refresh.clear_cache(id);
                    self.stack.remove_client(*id);
                }
            },
            GraphIdKind::Client => {
                if !self.connections.has_parents(id) {
                    self.refresh.clear_cache(id);
                    self.stack.remove_client(*id);
                }
            }
        }
    }

}
