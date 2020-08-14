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
};

struct DependenciesInner {
    refreshToken: HashMap<u64, RefreshToken>,
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

    pub fn triggerChange(&self, parentId: u64) {

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
                        log::error!("Coś poszło nie tak z pobieraniem refresh tokenów");
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

    //TODO - uprywatnić
    pub fn addRelation(&self, parentId: u64, refreshToken: RefreshToken) {
        self.inner.change((parentId, refreshToken), |state, (parentId, refreshToken)| {
            let clientId = refreshToken.getId();
            state.refreshToken.insert(clientId, refreshToken);

            state.graph.addRelation(parentId, clientId);
        });
    }

    //TODO - uprywatnić
    pub fn removeRelation(&self, clientId: u64) {
        self.inner.change(clientId, |state, clientId| {
            state.refreshToken.remove(&clientId);
            state.graph.removeRelation(clientId);
        });
    }

    pub fn startGetValueBlock(&self) {
        self.inner.changeNoParams(|state| {
            state.graph.startGetValueBlock();
        });
    }

    pub fn reportDependenceInStack(&self, parentId: u64) {
        self.inner.change(parentId, |state, parentId| {
            state.graph.reportDependenceInStack(parentId);
        });
    }

    pub fn endGetValueBlock(&self, computedRefresh: RefreshToken) {
        self.inner.change(computedRefresh, |state, computedRefresh| {
            let clientId = computedRefresh.getId();
            state.refreshToken.insert(clientId, computedRefresh);
            state.graph.endGetValueBlock(clientId);
        })
    }


    pub fn from<T: Debug, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        let deps = self.clone();

        let getValue = Box::new(move || {
            let result = calculate();

            Rc::new(result)
        });

        let result = Computed::new(deps, getValue);

        result
    }
}
