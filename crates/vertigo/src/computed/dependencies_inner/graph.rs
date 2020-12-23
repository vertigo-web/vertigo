use std::collections::{HashSet, HashMap, VecDeque};
use crate::computed::graph_id::GraphId;

struct GraphOne {
    rel: HashMap<GraphId, HashSet<GraphId>>,                //A <-> B
}

impl GraphOne {
    fn new() -> GraphOne {
        GraphOne {
            rel: HashMap::new()
        }
    }

    fn add(&mut self, edge_a: GraphId, edge_b: GraphId) {
        let list = self.rel.entry(edge_a).or_insert_with(HashSet::new);
        list.insert(edge_b);
    }


    pub fn remove_b(&mut self, edge_b: &GraphId) {
        self.rel.retain(|_k, list_ids| -> bool {

            list_ids.remove(edge_b);

            !list_ids.is_empty()
        });
    }

    pub fn get_all_deps(&self, edge_a: GraphId) -> HashSet<GraphId> {
        let mut result = HashSet::new();
        let mut to_traverse: Vec<GraphId> = vec!(edge_a);

        loop {
            let next_to_traverse = to_traverse.pop();

            match next_to_traverse {
                Some(next) => {
                    let list = self.rel.get(&next);

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
}

struct Stack {
    stack_relations: VecDeque<HashSet<GraphId>>,
}

impl Stack {
    fn new() -> Stack {
        Stack {
            stack_relations: VecDeque::new(),
        }
    }

    fn start_track(&mut self) {
        let stack_frame = HashSet::new();
        self.stack_relations.push_back(stack_frame);
    }

    fn report_dependence(&mut self, parent_id: GraphId) {
        let len = self.stack_relations.len();

        if len < 1 {
            log::warn!("frame with stack - not found len=0");
            return;
        }

        let last_index = len - 1;
        let last_item = self.stack_relations.get_mut(last_index);

        match last_item {
            Some(last_item) => {
                last_item.insert(parent_id);
            },
            None => {
                log::warn!("frame with stack - not found get_mut=None");
            }
        }
    }

    fn stop_track(&mut self) -> Option<HashSet<GraphId>> {
        self.stack_relations.pop_back()
    }
}


pub struct Graph {
    rel: GraphOne,                   //relacje parent <-> clientId
    //revert: GraphOne,                //relacje clientId <-> parent, wykorzystywane do powiadamiania o konieczności przeliczenia
    stack: Stack,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            rel: GraphOne::new(),
            //revert: GraphOne::new(),
            stack: Stack::new(),
        }
    }
}

impl Graph {
    fn add_relation(&mut self, parent_id: GraphId, client_id: GraphId) {
        self.rel.add(parent_id, client_id);
        //self.revert.add(clientId, parentId);
    }

    pub fn remove_relation(&mut self, client_id: &GraphId) {
        self.rel.remove_b(&client_id);
        //self.revert.removeA(clientId);
    }

    pub fn get_all_deps(&self, parent_id: GraphId) -> HashSet<GraphId> {
        self.rel.get_all_deps(parent_id)
    }

    pub fn report_dependence(&mut self, parent_id: GraphId) {
        self.stack.report_dependence(parent_id);
    }

    pub fn start_track(&mut self) {
        self.stack.start_track();
    }

    pub fn stop_track(&mut self, client_id: GraphId) {

        let last_item = self.stack.stop_track();

        match last_item {
            Some(last_item) => {

                for parent_id in last_item {
                    self.add_relation(parent_id, client_id.clone());
                }
            },
            None => {
                panic!("TODO - Spodziewano się elementu");
            }
        }
    }
}
