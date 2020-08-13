use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    Graph::Graph,
    Value::Value,
    Client::ClientRefresh,
    Computed::ComputedRefresh,
};

struct DependenciesInner {
    computed: HashMap<u64, ComputedRefresh>,        //To wykorzystujemy do wytrigerowania odpowiednich akcji
    client: HashMap<u64, ClientRefresh>,            //To wykorzystujemy do wytrigerowania odpowiedniej reakcji
    graph: Graph,
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            computed: HashMap::new(),
            client: HashMap::new(),
            graph: Graph::new(),
        }
    }
}

pub struct Dependencies {
    inner: Rc<BoxRefCell<DependenciesInner>>,
}

impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Dependencies {
            inner: self.inner.clone()
        }
    }
}

impl Dependencies {
    pub fn new() -> Dependencies {        
        Dependencies {
            inner: Rc::new(BoxRefCell::new(DependenciesInner::new()))
        }
    }

    pub fn newValue<T: Debug>(&self, value: T) -> Value<T> {
        Value::new(self.clone(), value)
    }

    pub fn triggerChange(&self, parentId: u64) {

        self.inner.getWithContext(parentId, |state, parentId| {
            let allDeps = state.graph.getAllDeps(parentId);

            for itemId in allDeps.iter() {
                let item = state.computed.get(itemId);

                if let Some(item) = item {
                    item.setAsUnfreshInner();
                }
            }

            for itemId in allDeps.iter() {
                let item = state.client.get(itemId);

                if let Some(item) = item {
                    item.recalculate();
                }
            }
        });
    }

    pub fn addRelation(&self, parentId: u64, client: ComputedRefresh) {
        self.inner.change((parentId, client), |state, (parentId, client)| {
            let clientId = client.getId();
            state.computed.insert(clientId, client);

            state.graph.addRelation(parentId, clientId);
        });
    }

    pub fn addRelationToClient(&self, parentId: u64, client: ClientRefresh) {
        self.inner.change((parentId, client), |state, (parentId, client)| {
            let clientId = client.getId();
            state.client.insert(clientId, client);

            state.graph.addRelation(parentId, clientId);
        });
    }

    pub fn removeRelation(&self, clientId: u64) {
        self.inner.change(clientId, |state, clientId| {
            state.computed.remove(&clientId);
            state.client.remove(&clientId);

            state.graph.removeRelation(clientId);
        });
    }
}
