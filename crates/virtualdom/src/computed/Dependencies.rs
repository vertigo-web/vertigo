use std::collections::{HashMap};
use std::rc::Rc;
use std::collections::HashSet;

use crate::computed::{
    BoxRefCell::BoxRefCell,
    Graph::Graph,
    Value::Value,
    RefreshToken::RefreshToken,
    Computed::{
        Computed,
    },
    GraphId::GraphId,
};

enum TransactionState {
    Idle,
    Modification {                          //Modifying the first layer
        level: u16,                         //current transacion level
        edges: HashSet<GraphId>,            //edges to refresh
    },
    Refreshing
}

impl TransactionState {
    pub fn up(&mut self) -> bool {
        match self {
            TransactionState::Idle => {
                *self = TransactionState::Modification {
                    level: 1,
                    edges: HashSet::new()
                };

                true
            },
            TransactionState::Modification { level, .. } => {
                *level += 1;
                true
            },
            TransactionState::Refreshing => {
                log::error!("You cannot change the source value while the dependency graph is being refreshed");
                false
            }
        }
    }

    pub fn down(&mut self) -> Option<HashSet<GraphId>> {
        match self {
            TransactionState::Idle => {
                log::error!("You cannot call 'down' for a state 'TransactionState::Idle'");

                None
            },
            TransactionState::Modification { level, edges } => {
                *level -= 1;

                if *level == 0 {
                    let edges = std::mem::replace(edges, HashSet::new());
                    *self = TransactionState::Refreshing;
                    return Some(edges);
                }

                None
            },
            TransactionState::Refreshing => {
                log::error!("You cannot change the source value while the dependency graph is being refreshed");
                None
            }
        }
    }

    pub fn to_idle(&mut self) {
        match self {
            TransactionState::Idle => {
                log::error!("you cannot go from 'TransactionState::Idle' to 'TransactionState::Idle'");
            },
            TransactionState::Modification { .. } => {
                log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
            },
            TransactionState::Refreshing => {
                *self = TransactionState::Idle;
            }
        }
    }

    pub fn add_edges_to_refresh(&mut self, mut new_edges: HashSet<GraphId>) {
        match self {
            TransactionState::Modification { edges, .. } => {
                for id in new_edges.drain() {
                    edges.insert(id);
                }
            },
            _ => {
                log::error!("You can only call the trigger if you are in a transaction block");
            }
        }
    }
}

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
