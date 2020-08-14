use std::collections::{HashSet, HashMap, VecDeque};

pub struct Graph {
    relations: HashMap<u64, u64>,                   //relacje zaleności, target -> parent
    revertRelations: HashMap<u64, HashSet<u64>>,        //wykorzystywane do powiadamiania o konieczności przeliczenia
                                                    //parent -> Vec<target>

    stackRelations: VecDeque<HashSet<u64>>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            relations: HashMap::new(),
            revertRelations: HashMap::new(),
            stackRelations: VecDeque::new(),
        }
    }

    pub fn addRelation(&mut self, parentId: u64, clientId: u64) {
        self.relations.insert(clientId, parentId);

        let list = self.revertRelations.entry(parentId).or_insert_with(HashSet::new);
        list.insert(clientId);
    }

    pub fn removeRelation(&mut self, clientId: u64) {
        let parentId = self.relations.remove(&clientId);

        let parentId = match parentId {
            Some(parentId) => parentId,
            None => {
                log::error!("relationship delete error - first level");
                return;
            }
        };

        let listIds = self.revertRelations.get_mut(&parentId);

        let listIds = match listIds {
            Some(listIds) => listIds,
            None => {
                log::error!("relationship delete error - second level");
                return;
            }
        };

        let removed = listIds.remove(&clientId);

        if removed == false {
            log::error!("relationship delete error - missing item");
        }
    }

    pub fn getAllDeps(&self, parentId: u64) -> HashSet<u64> {
        let mut result = HashSet::new();
        let mut toTraverse: Vec<u64> = vec!(parentId);

        loop {
            let nextToTraverse = toTraverse.pop();

            match nextToTraverse {
                Some(next) => {
                    result.insert(next);

                    let list = self.revertRelations.get(&next);

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

    pub fn startGetValueBlock(&mut self) {
        let stackFrame = HashSet::new();
        self.stackRelations.push_back(stackFrame);
    }

    pub fn reportDependenceInStack(&mut self, parentId: u64) {
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

    pub fn endGetValueBlock(&mut self, clientId: u64) {

        let lastItem = self.stackRelations.pop_back();

        match lastItem {
            Some(lastItem) => {
                

                //TODO - usunac nieuzywane krawedzie (subskrybcje)


                for parentId in lastItem {
                    self.addRelation(parentId, clientId);
                }
                //todo!("tutaj trzeba obsluzyc te zaleznosci")
            },
            None => {
                panic!("TODO - Spodziewano się elementu");
            }
        }
    }

    // pub fn computed<F>() -> R {}
}
