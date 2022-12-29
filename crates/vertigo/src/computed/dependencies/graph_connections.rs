use std::{collections::{BTreeSet, BTreeMap}};
use crate::{GraphId, struct_mut::ValueMut};
use super::graph_one_to_many::GraphOneToMany;

enum UpdateConnection {
    Add {
        parent: GraphId,
        client: GraphId
    },
    Remove {
        parent: GraphId,
        client: GraphId
    },
}

impl UpdateConnection {
    fn get_parent(&self) -> GraphId {
        match self {
            Self::Add { parent, .. } => *parent,
            Self::Remove { parent, .. } => *parent
        }
    }
}

struct GraphConnectionsInner {
    parent_client: GraphOneToMany,                    // ParentId <- ClientId
    client_parent: GraphOneToMany,
}

impl GraphConnectionsInner {
    pub fn new() -> GraphConnectionsInner {
        GraphConnectionsInner {
            parent_client: GraphOneToMany::new(),
            client_parent: GraphOneToMany::new(),
        }
    }

    fn exec_command(&mut self, command: UpdateConnection) -> Vec<UpdateConnection> {
        match command {
            UpdateConnection::Add { parent, client } => {
                self.parent_client.add(parent, client);
                self.client_parent.add(client, parent);
                Vec::new()
            },
            UpdateConnection::Remove { parent, client } => {
                self.parent_client.remove(parent, client);
                self.client_parent.remove(client, parent);

                let child = self.parent_client.get_relation(parent);

                if child.is_empty() {
                    self.client_parent.get_relation(parent)
                        .map(|parent_next| UpdateConnection::Remove { parent: parent_next, client: parent })
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }


    fn exec_command_list(&mut self, command_list: Vec<UpdateConnection>) -> Vec<UpdateConnection> {
        let mut new_list = Vec::new();

        for command in command_list {
            new_list.extend(self.exec_command(command).into_iter());
        }

        new_list
    }

    ///The function receives new parents. Calculates the updates that should be applied to the graph
    fn get_update_commands(&self, client_id: GraphId, new_parents: BTreeSet<GraphId>) -> Vec<UpdateConnection> {
        let prev_parents = self
            .client_parent
            .get_relation(client_id)
            .collect::<BTreeSet<_>>();

        let mut edge_on: Vec<UpdateConnection> = new_parents
            .difference(&prev_parents)
            .map(|parent| UpdateConnection::Add { parent: *parent, client: client_id })
            .collect();

        let edge_off = prev_parents
            .difference(&new_parents)
            .map(|parent| UpdateConnection::Remove { parent: *parent, client: client_id });

        edge_on.extend(edge_off);
        edge_on
    }

    fn get_info_about_active(&self, nodes_for_refresh: BTreeSet<GraphId>) -> BTreeMap<GraphId, bool> {
        let mut result = BTreeMap::new();

        for client_id in nodes_for_refresh {
            let node_is_active = !self.parent_client.get_relation(client_id).is_empty();
            result.insert(client_id, node_is_active);
        }

        result
    }

    fn set_parent_for_client(&mut self, client_id: GraphId, new_parents: BTreeSet<GraphId>) -> BTreeMap<GraphId, bool> {
        let mut nodes_for_refresh: BTreeSet<GraphId> = BTreeSet::new();
        nodes_for_refresh.insert(client_id);

        let mut commands = self.get_update_commands(client_id, new_parents);

        loop {
            for command in commands.iter() {
                nodes_for_refresh.insert(command.get_parent());
            }

            commands = self.exec_command_list(commands);

            if commands.is_empty() {
                return self.get_info_about_active(nodes_for_refresh);
            }
        }
    }

    pub(crate) fn get_all_deps(&self, id: GraphId) -> BTreeSet<GraphId> {
        let mut result = BTreeSet::new();
        let mut to_traverse: Vec<GraphId> = vec!(id);

        loop {
            let Some(next) = to_traverse.pop() else {
                return result;
            };

            for item in self.parent_client.get_relation(next) {
                let is_added = result.insert(item);
                if is_added {
                    to_traverse.push(item);
                }
            }
        }
    }

    #[cfg(test)]
    fn all_connections_len(&self) -> u64 {
        self.parent_client.all_connections_len()
    }

    // pub fn debug(&self) {
    //     log::info!("\n\n");
    //     log::info!("--- debug start ---");
    //     for (parent, client_ids) in self.parent_client.iter() {
    //         log::info!("{parent:?}");
    //         for client_id in client_ids {
    //             log::info!(" - {client_id:?}");
    //         }
    //     }

    //     log::info!("--- debug end ---");
    //     log::info!("\n\n");
    // }
}


/*
parent_client - to powinno być czysto reaktywne, ustawianie czegoś w "client_parent" powinno powodować updejt "parent_client"

usunięcie jakiegoś client_id, powinno spowodować, ze rekurencyjnie zostaną usunięte niepotrzebne referencje do parentów
*/


pub struct GraphConnections {
    inner: ValueMut<GraphConnectionsInner>,
}

impl GraphConnections {
    pub fn new() -> GraphConnections {
        GraphConnections { 
            inner: ValueMut::new(GraphConnectionsInner::new()),
        }
    }

    pub(crate) fn set_parent_for_client(&self, client_id: GraphId, parents_list: BTreeSet<GraphId>) -> BTreeMap<GraphId, bool> {
        self.inner.change(move |state| {
            state.set_parent_for_client(client_id, parents_list)
        })
    }

    pub(crate) fn get_all_deps(&self, id: GraphId) -> BTreeSet<GraphId> {
        self.inner.change(|state| state.get_all_deps(id))
    }

    #[cfg(test)]
    pub(crate) fn all_connections_len(&self) -> u64 {
        self.inner.map(|state| state.all_connections_len())
    }
}
