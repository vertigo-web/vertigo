use std::collections::{HashSet, HashMap, VecDeque};

struct GraphOne {
    rel: HashMap<u64, HashSet<u64>>,                //A <-> B
}

impl GraphOne {
    fn new() -> GraphOne {
        GraphOne {
            rel: HashMap::new()
        }
    }

    fn add(&mut self, edgeA: u64, edgeB: u64) {
        let list = self.rel.entry(edgeA).or_insert_with(HashSet::new);
        list.insert(edgeB);
    }

    #[allow(dead_code)]
    pub fn removeA(&mut self, edgeA: u64) {
        self.rel.remove(&edgeA);
    }

    pub fn removeB(&mut self, edgeB: u64) {
        self.rel.retain(|_k, listIds| -> bool {

            listIds.remove(&edgeB);

            listIds.len() > 0
        });
    }

    pub fn getAllDeps(&self, edgeA: u64) -> HashSet<u64> {
        let mut result = HashSet::new();
        let mut toTraverse: Vec<u64> = vec!(edgeA);

        loop {
            let nextToTraverse = toTraverse.pop();

            match nextToTraverse {
                Some(next) => {
                    result.insert(next);

                    let list = self.rel.get(&next);

                    if let Some(list) = list {

                        for item in list {
                            let isContain = result.contains(item);
                            if isContain {
                                //ignore
                            } else {

                                toTraverse.push(*item);
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
    stackRelations: VecDeque<HashSet<u64>>,
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

    fn reportDependence(&mut self, parentId: u64) {
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

    fn stopTrack(&mut self) -> Option<HashSet<u64>> {
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

    fn addRelation(&mut self, parentId: u64, clientId: u64) {
        self.rel.add(parentId, clientId);
        //self.revert.add(clientId, parentId);
    }

    pub fn removeRelation(&mut self, clientId: u64) {
        self.rel.removeB(clientId);
        //self.revert.removeA(clientId);
    }

    pub fn getAllDeps(&self, parentId: u64) -> HashSet<u64> {
        self.rel.getAllDeps(parentId)
    }

    pub fn reportDependence(&mut self, parentId: u64) {
        self.stack.reportDependence(parentId);
    }

    pub fn startTrack(&mut self) {
        self.stack.startTrack();
    }

    pub fn stopTrack(&mut self, clientId: u64) {

        let lastItem = self.stack.stopTrack();

        match lastItem {
            Some(lastItem) => {

                for parentId in lastItem {
                    self.addRelation(parentId, clientId);
                }
            },
            None => {
                panic!("TODO - Spodziewano się elementu");
            }
        }
    }
}
