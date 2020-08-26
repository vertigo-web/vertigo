use std::collections::{HashSet, HashMap, VecDeque};
use crate::lib::GraphId::GraphId;

struct GraphOne {
    rel: HashMap<GraphId, HashSet<GraphId>>,                //A <-> B
}

impl GraphOne {
    fn new() -> GraphOne {
        GraphOne {
            rel: HashMap::new()
        }
    }

    fn add(&mut self, edgeA: GraphId, edgeB: GraphId) {
        let list = self.rel.entry(edgeA).or_insert_with(HashSet::new);
        list.insert(edgeB);
    }

    #[allow(dead_code)]
    pub fn removeA(&mut self, edgeA: GraphId) {
        self.rel.remove(&edgeA);
    }

    pub fn removeB(&mut self, edgeB: &GraphId) {
        self.rel.retain(|_k, listIds| -> bool {

            listIds.remove(edgeB);

            listIds.len() > 0
        });
    }

    pub fn getAllDeps(&self, edgeA: GraphId) -> HashSet<GraphId> {
        let mut result = HashSet::new();
        let mut toTraverse: Vec<GraphId> = vec!(edgeA.clone());

        loop {
            let nextToTraverse = toTraverse.pop();

            match nextToTraverse {
                Some(next) => {
                    if next != edgeA {
                        result.insert(next.clone());
                    }

                    let list = self.rel.get(&next);

                    if let Some(list) = list {

                        for item in list {
                            let isContain = result.contains(item);
                            if isContain {
                                //ignore
                            } else {

                                toTraverse.push(item.clone());
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
    stackRelations: VecDeque<HashSet<GraphId>>,
}

impl Stack {
    fn new() -> Stack {
        Stack {
            stackRelations: VecDeque::new(),
        }
    }

    fn startTrack(&mut self) {
        let stackFrame = HashSet::new();
        self.stackRelations.push_back(stackFrame);
    }

    fn reportDependence(&mut self, parentId: GraphId) {
        let len = self.stackRelations.len();

        if len < 1 {
            log::warn!("frame with stack - not found len=0");
            return;
        }

        let lastIndex = len - 1;
        let lastItem = self.stackRelations.get_mut(lastIndex);

        match lastItem {
            Some(lastItem) => {
                lastItem.insert(parentId);
            },
            None => {
                log::warn!("frame with stack - not found get_mut=None");
            }
        }
    }

    fn stopTrack(&mut self) -> Option<HashSet<GraphId>> {
        self.stackRelations.pop_back()
    }
}


pub struct Graph {
    rel: GraphOne,                   //relacje parent <-> clientId
    //revert: GraphOne,                //relacje clientId <-> parent, wykorzystywane do powiadamiania o konieczności przeliczenia
    stack: Stack,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            rel: GraphOne::new(),
            //revert: GraphOne::new(),
            stack: Stack::new(),
        }
    }

    fn addRelation(&mut self, parentId: GraphId, clientId: GraphId) {
        self.rel.add(parentId, clientId);
        //self.revert.add(clientId, parentId);
    }

    pub fn removeRelation(&mut self, clientId: &GraphId) {
        self.rel.removeB(&clientId);
        //self.revert.removeA(clientId);
    }

    pub fn getAllDeps(&self, parentId: GraphId) -> HashSet<GraphId> {
        self.rel.getAllDeps(parentId)
    }

    pub fn reportDependence(&mut self, parentId: GraphId) {
        self.stack.reportDependence(parentId);
    }

    pub fn startTrack(&mut self) {
        self.stack.startTrack();
    }

    pub fn stopTrack(&mut self, clientId: GraphId) {

        let lastItem = self.stack.stopTrack();

        match lastItem {
            Some(lastItem) => {

                for parentId in lastItem {
                    self.addRelation(parentId, clientId.clone());
                }
            },
            None => {
                panic!("TODO - Spodziewano się elementu");
            }
        }
    }
}
