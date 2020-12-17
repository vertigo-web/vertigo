use std::collections::{HashMap};
use std::rc::Rc;
use std::collections::HashSet;

use crate::computed::{
    BoxRefCell,
    Value,
    refresh_token::RefreshToken,
    Computed,
    GraphId::GraphId,
};

use super::dependencies_inner::{Graph::Graph, TransactionState::TransactionState};

struct DependenciesInner {
    refreshToken: HashMap<GraphId, RefreshToken>,               //Tablica z zarejestrowanymi tokenami
    graph: Graph,
    transactionState: TransactionState,              //aktualny poziom tranzakcyjności
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            refreshToken: HashMap::new(),
            graph: Graph::default(),
            transactionState: TransactionState::Idle,
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

impl Default for Dependencies {
    fn default() -> Self {
        Self {
            inner: Rc::new(BoxRefCell::new(DependenciesInner::new()))
        }
    }
}

impl Dependencies {
    pub fn newValue<T>(&self, value: T) -> Value<T> {
        Value::new(self.clone(), value)
    }

    pub fn newComputedFrom<T>(&self, value: T) -> Computed<T> {
        let value = self.newValue(value);
        value.toComputed()
    }

    fn refresh_edges(&self, edges: HashSet<GraphId>) {
        let refreshToken: Vec<RefreshToken> = self.inner.getWithContext(
            edges,
            |state, edges| {
                let mut out: Vec<RefreshToken> = Vec::new();

                for itemId in edges.iter() {
                    let item = state.refreshToken.get(itemId);

                    if let Some(item) = item {
                        out.push(item.clone());
                    } else {
                        log::error!("Coś poszło nie tak z pobieraniem refresh tokenów {:?}", itemId);
                    }
                }

                out
            }
        );

        let mut clientRefreshToken = Vec::new();

        for item in refreshToken.iter() {
            if item.isComputed() {
                item.update();
            } else {
                clientRefreshToken.push(item);
            }
        }

        for item in clientRefreshToken {
            item.update();
        }
    }

    pub fn transaction<F: FnOnce()>(&self, func: F) {
        let success = self.inner.changeNoParams(|state| {
            state.transactionState.up()
        });

        if !success {
            return;
        }

        func();

        let edges_for_refresh = self.inner.changeNoParams(|state| {
            state.transactionState.down()
        });

        if let Some(edges) = edges_for_refresh {
            self.refresh_edges(edges);

            self.inner.changeNoParams(|state| {
                state.transactionState.to_idle()
            });
        }
    }

    pub fn triggerChange(&self, parentId: GraphId) {
        self.inner.change(parentId, |state, parentId| {
            let edges = state.graph.getAllDeps(parentId);
            state.transactionState.add_edges_to_refresh(edges);
        });
    }

    pub fn registerRefreshToken(&self, clientId: GraphId, refreshToken: RefreshToken) {
        self.inner.change(
            (clientId, refreshToken),
            |state, (clientId, refreshToken)| {
                state.refreshToken.insert(clientId, refreshToken);
            }
        );
    }

    pub fn removeRelation(&self, clientId: &GraphId) -> Option<RefreshToken> {
        self.inner.change(clientId, |state, clientId| {
            let refreshToken = state.refreshToken.remove(&clientId);
            state.graph.removeRelation(&clientId);
            refreshToken
        })
    }

    fn startTrack(&self) {
        self.inner.changeNoParams(|state| {
            state.graph.startTrack();
        });
    }

    fn stopTrack(&self, clientId: GraphId) {
        self.inner.change(clientId, |state, clientId| {
            state.graph.stopTrack(clientId);
        })
    }

    pub fn reportDependenceInStack(&self, parentId: GraphId) {
        self.inner.change(parentId, |state, parentId| {
            state.graph.reportDependence(parentId);
        });
    }

    pub fn wrapGetValue<T, F: Fn() -> T + 'static>(&self, getValue: F, clientId: GraphId) -> Box<dyn Fn() -> T> {
        let selfClone = self.clone();

        Box::new(move || {

            selfClone.startTrack();

            let result = getValue();

            selfClone.stopTrack(clientId.clone());

            result
        })
    }

    pub fn from<T: 'static, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        let deps = self.clone();

        let getValue = Box::new(move || {
            let result = calculate();

            Rc::new(result)
        });

        Computed::new(deps, getValue)
    }
}
