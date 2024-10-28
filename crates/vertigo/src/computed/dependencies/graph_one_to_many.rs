use crate::GraphId;
use std::collections::{BTreeMap, BTreeSet};

pub struct GraphEdgeIter<'a> {
    data: Option<&'a BTreeSet<GraphId>>,
    data_iter: Option<std::collections::btree_set::Iter<'a, GraphId>>,
}

impl<'a> GraphEdgeIter<'a> {
    pub fn new(data: Option<&'a BTreeSet<GraphId>>) -> GraphEdgeIter<'a> {
        let data_iter = data.map(|item| item.iter());

        Self { data, data_iter }
    }

    pub fn is_empty(&self) -> bool {
        match self.data {
            Some(edges) => edges.is_empty(),
            None => true,
        }
    }
}

impl Iterator for GraphEdgeIter<'_> {
    type Item = GraphId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(data_iter) = &mut self.data_iter {
            let next = data_iter.next();
            next.cloned()
        } else {
            None
        }
    }
}

pub struct GraphOneToMany {
    data: BTreeMap<GraphId, BTreeSet<GraphId>>,
}

impl GraphOneToMany {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, left_id: GraphId, right_id: GraphId) {
        self.data.entry(left_id).or_default().insert(right_id);
    }

    pub fn remove(&mut self, parent_id: GraphId, client_id: GraphId) {
        let should_clear = match self.data.get_mut(&parent_id) {
            Some(parent_list) => {
                parent_list.remove(&client_id);
                parent_list.is_empty()
            }
            None => false,
        };

        if should_clear {
            self.data.remove(&parent_id);
        }
    }

    pub fn get_relation(&self, left_id: GraphId) -> GraphEdgeIter<'_> {
        let edge = self.data.get(&left_id);
        GraphEdgeIter::new(edge)
    }

    #[cfg(test)]
    pub fn all_connections_len(&self) -> u64 {
        let mut count: u64 = 0;

        for (_, item) in self.data.iter() {
            count += item.len() as u64;
        }

        count
    }
}
