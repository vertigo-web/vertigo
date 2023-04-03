use std::collections::{BTreeSet, BTreeMap};
use crate::DomId;
use super::dom_connection::DomConnection;

pub struct NodePaths {
    paths: BTreeMap<DomId, BTreeSet<DomId>>,
    all_nodes: BTreeSet<DomId>,
}

impl NodePaths {
    pub fn new() -> Self {
        Self {
            paths: BTreeMap::new(),
            all_nodes: BTreeSet::new(),
        }
    }

    pub fn refresh_paths(&mut self, dom_connection: &DomConnection) {
        for (suspense_id, suspense_path) in self.paths.iter_mut() {
            *suspense_path = calculate_path(dom_connection, *suspense_id);
        }

        self.all_nodes = calculate_all_nodes(&self.paths);
    }

    #[must_use]
    pub fn remove(&mut self, node_id: DomId) -> bool {
        let item = self.paths.remove(&node_id);

        let is_removed = item.is_some();

        if is_removed {
            self.all_nodes = calculate_all_nodes(&self.paths);
        }

        is_removed
    }

    pub fn insert(&mut self, dom_connection: &DomConnection, node_id: DomId) {
        self.paths.insert(node_id, calculate_path(dom_connection, node_id));
        self.all_nodes = calculate_all_nodes(&self.paths);
    }

    pub fn contains(&self, node_id: &DomId) -> bool {
        self.all_nodes.contains(node_id)
    }
}


fn calculate_path(dom_connection: &DomConnection, suspense_id: DomId) -> BTreeSet<DomId> {
    let mut path = BTreeSet::new();

    let mut current = suspense_id;
    path.insert(current);

    let mut max_level = 500;

    loop {
        max_level -= 1;

        if max_level < 0 {
            panic!("Too much nesting or recursion error");
        }

        if let Some(parent) = dom_connection.get_parent(current) {
            current = parent;
            path.insert(current);
        } else {
            return path;
        }
    }
}

fn calculate_all_nodes(paths: &BTreeMap<DomId, BTreeSet<DomId>>) -> BTreeSet<DomId> {
    let mut all_node = BTreeSet::new();

    for (_, path) in paths.iter() {
        for node_id in path {
            all_node.insert(*node_id);
        }
    }
    all_node
}
