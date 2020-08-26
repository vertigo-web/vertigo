use std::collections::{HashMap};
use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    Graph::Graph,
    Value::Value,
    RefreshToken::RefreshToken,
    Computed::{
        Computed,
    },
    GraphId::GraphId,
};

struct DependenciesInner {
    refreshToken: HashMap<GraphId, RefreshToken>,               //Tablica z zarejestrowanymi tokenami
    graph: Graph,
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            refreshToken: HashMap::new(),
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

    pub fn triggerChange(&self, parentId: GraphId) {

        let refreshToken: Vec<RefreshToken> = self.inner.getWithContext(
            parentId,
            |state, parentId| {
                let allDeps = state.graph.getAllDeps(parentId);

                let mut out: Vec<RefreshToken> = Vec::new();

                for itemId in allDeps.iter() {
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

    pub fn registerRefreshToken(&self, clientId: GraphId, refreshToken: RefreshToken) {
        self.inner.change(
            (clientId, refreshToken),
            |state, (clientId, refreshToken)| {
                state.refreshToken.insert(clientId.clone(), refreshToken);
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

        let result = Computed::new(deps, getValue);

        result
    }
}
