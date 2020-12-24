use std::collections::{HashMap};
use std::rc::Rc;
use std::collections::HashSet;

use crate::computed::{
    BoxRefCell,
    Value,
    refresh_token::RefreshToken,
    Computed,
    GraphId,
};

use super::dependencies_inner::{graph::Graph, transaction_state::TransactionState};

struct DependenciesInner {
    refresh_token: HashMap<GraphId, RefreshToken>,               //Tablica z zarejestrowanymi tokenami
    graph: Graph,
    transaction_state: TransactionState,              //aktualny poziom tranzakcyjności
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            refresh_token: HashMap::new(),
            graph: Graph::default(),
            transaction_state: TransactionState::Idle,
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
    pub fn new_value<T>(&self, value: T) -> Value<T> {
        Value::new(self.clone(), value)
    }

    pub fn new_computed_from<T>(&self, value: T) -> Computed<T> {
        let value = self.new_value(value);
        value.to_computed()
    }

    fn refresh_edges(&self, edges: HashSet<GraphId>) {
        let refresh_token: Vec<RefreshToken> = self.inner.get_with_context(
            edges,
            |state, edges| {
                let mut out: Vec<RefreshToken> = Vec::new();

                for item_id in edges.iter() {
                    let item = state.refresh_token.get(item_id);

                    if let Some(item) = item {
                        out.push(item.clone());
                    } else {
                        log::error!("Coś poszło nie tak z pobieraniem refresh tokenów {:?}", item_id);
                    }
                }

                out
            }
        );

        let mut client_refresh_token = Vec::new();

        for item in refresh_token.iter() {
            if item.is_computed() {
                item.update();
            } else {
                client_refresh_token.push(item);
            }
        }

        for item in client_refresh_token {
            item.update();
        }
    }

    pub fn transaction<F: FnOnce()>(&self, func: F) {
        let success = self.inner.change_no_params(|state| {
            state.transaction_state.up()
        });

        if !success {
            return;
        }

        func();

        let edges_for_refresh = self.inner.change_no_params(|state| {
            state.transaction_state.down()
        });

        if let Some(edges) = edges_for_refresh {
            self.refresh_edges(edges);

            self.inner.change_no_params(|state| {
                state.transaction_state.to_idle()
            });
        }
    }

    pub fn trigger_change(&self, parent_id: GraphId) {
        self.inner.change(parent_id, |state, parent_id| {
            let edges = state.graph.get_all_deps(parent_id);
            state.transaction_state.add_edges_to_refresh(edges);
        });
    }

    pub fn register_refresh_token(&self, client_id: GraphId, refresh_token: RefreshToken) {
        self.inner.change(
            (client_id, refresh_token),
            |state, (client_id, refresh_token)| {
                state.refresh_token.insert(client_id, refresh_token);
            }
        );
    }

    pub fn remove_relation(&self, client_id: &GraphId) -> Option<RefreshToken> {
        self.inner.change(client_id, |state, client_id| {
            let refresh_token = state.refresh_token.remove(&client_id);
            state.graph.remove_relation(&client_id);
            refresh_token
        })
    }

    fn start_track(&self) {
        self.inner.change_no_params(|state| {
            state.graph.start_track();
        });
    }

    fn stop_track(&self, client_id: GraphId) {
        self.inner.change(client_id, |state, client_id| {
            state.graph.stop_track(client_id);
        })
    }

    pub fn report_dependence_in_stack(&self, parent_id: GraphId) {
        self.inner.change(parent_id, |state, parent_id| {
            state.graph.report_dependence(parent_id);
        });
    }

    pub fn wrap_get_value<T, F: Fn() -> T + 'static>(&self, get_value: F, client_id: GraphId) -> Box<dyn Fn() -> T> {
        let self_clone = self.clone();

        Box::new(move || {

            self_clone.start_track();

            let result = get_value();

            self_clone.stop_track(client_id.clone());

            result
        })
    }

    pub fn from<T: 'static, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        let deps = self.clone();

        let get_value = Box::new(move || {
            let result = calculate();

            Rc::new(result)
        });

        Computed::new(deps, get_value)
    }
}
