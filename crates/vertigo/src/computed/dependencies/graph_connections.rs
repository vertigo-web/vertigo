use std::collections::{BTreeSet, BTreeMap};

use crate::{GraphId, struct_mut::ValueMut};

fn add_connection(data: &mut BTreeMap<GraphId, BTreeSet<GraphId>>, parent_list: &BTreeSet<GraphId>, client_id: GraphId) {
    if parent_list.is_empty() {
        return;
    }

    for parent_id in parent_list {
        data
            .entry(*parent_id)
            .or_insert_with(BTreeSet::new)
            .insert(client_id);
    }
}

fn remove_connection(data: &mut BTreeMap<GraphId, BTreeSet<GraphId>>, parent_list: &BTreeSet<GraphId>, client_id: GraphId) {
    for parent_id in parent_list {

        let should_clear = if let Some(parent_list) = data.get_mut(parent_id) {
            parent_list.remove(&client_id);
            parent_list.is_empty()
        } else {
            false
        };

        if should_clear {
            data.remove(parent_id);
        }
    }
}

struct GraphConnectionsInner {
    parent_client: BTreeMap<GraphId, BTreeSet<GraphId>>,                    // ParentId <- ClientId
    client_parent: BTreeMap<GraphId, BTreeSet<GraphId>>,
}

impl GraphConnectionsInner {
    pub fn new() -> GraphConnectionsInner {
        GraphConnectionsInner {
            parent_client: BTreeMap::new(),
            client_parent: BTreeMap::new(),
        }
    }

    fn set_parent_for_client(&mut self, new_parents: BTreeSet<GraphId>, client_id: GraphId) -> Option<BTreeSet<GraphId>> {
        let prev_parents = self.client_parent.remove(&client_id);

        let off_edges = match prev_parents {
            Some(prev_parents) => {
                let edge_on = new_parents.difference(&prev_parents).copied().collect();
                let edge_off: BTreeSet<GraphId> = prev_parents.difference(&new_parents).copied().collect();

                add_connection(&mut self.parent_client, &edge_on, client_id);
                remove_connection(&mut self.parent_client, &edge_off, client_id);

                Some(edge_off)
            },
            None => {
                add_connection(&mut self.parent_client, &new_parents, client_id);
                None
            }
        };

        if !new_parents.is_empty() {
            self.client_parent.insert(client_id, new_parents);
        }

        off_edges
    }

    pub fn has_subscribers(&self, id: &GraphId) -> bool {
        if let Some(item) = self.parent_client.get(id) {
            !item.is_empty()
        } else {
            false
        }
    }

    pub fn has_parents(&self, id: &GraphId) -> bool {
        if let Some(item) = self.client_parent.get(id) {
            !item.is_empty()
        } else {
            false
        }
    }

    pub(crate) fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        let mut result = BTreeSet::new();
        let mut to_traverse: Vec<GraphId> = edges.into_iter().collect();

        loop {
            let next_to_traverse = to_traverse.pop();

            match next_to_traverse {
                Some(next) => {
                    let list = self.parent_client.get(&next);

                    if let Some(list) = list {
                        for item in list {
                            let is_added = result.insert(*item);
                            if is_added {
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
    }

    fn all_connections_len(&self) -> u64 {
        let mut count: u64 = 0;

        for (_, item) in self.parent_client.iter()  {
            count += item.len() as u64;
        }

        count
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

    pub(crate) fn set_parent_for_client(&self, parents_list: BTreeSet<GraphId>, client_id: GraphId) -> Option<BTreeSet<GraphId>> {
        self.inner.change(move |state| {
            state.set_parent_for_client(parents_list, client_id)
        })
    }

    pub(crate) fn has_subscribers(&self, id: &GraphId) -> bool {
        self.inner.map(|state| state.has_subscribers(id))
    }

    pub(crate) fn has_parents(&self, id: &GraphId) -> bool {
        self.inner.map(|state| state.has_parents(id))
    }

    pub(crate) fn get_all_deps(&self, edges: BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        self.inner.change(|state| state.get_all_deps(edges))
    }

    pub(crate) fn all_connections_len(&self) -> u64 {
        self.inner.map(|state| state.all_connections_len())
    }
}
