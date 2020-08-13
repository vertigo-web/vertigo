use std::collections::{HashSet, HashMap};

pub struct Graph {
    relations: HashMap<u64, u64>,                   //relacje zaleności, target -> parent
    revertRelations: HashMap<u64, Vec<u64>>,        //wykorzystywane do powiadamiania o konieczności przeliczenia
                                                    //parent -> Vec<target>
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            relations: HashMap::new(),
            revertRelations: HashMap::new(),
        }
    }

    pub fn addRelation(&mut self, parentId: u64, clientId: u64) {
        self.relations.insert(clientId, parentId);

        let list = self.revertRelations.entry(parentId).or_insert_with(Vec::new);
        list.push(clientId);
    }

    pub fn removeRelation(&mut self, clientId: u64) {
        self.relations.remove(&clientId);
        self.revertRelations.retain(|_k, listIds| {
            listIds.retain(|item| {
                let matchId = clientId == *item;
                let shouldStay = !matchId;
                shouldStay
            });

            listIds.len() > 0
        });
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
}
