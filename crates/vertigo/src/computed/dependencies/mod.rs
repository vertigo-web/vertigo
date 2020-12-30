use std::rc::Rc;
use std::cmp::PartialEq;
use std::collections::BTreeSet;

use crate::computed::{
    BoxRefCell,
    Value,
    Computed,
    GraphId,
    EqBox,
    GraphValueRefresh,
};

mod graph;
mod graph_map;
mod transaction_state;
mod stack;
mod refresh_edges;

use {
    graph::Graph,
    stack::Stack,
    transaction_state::TransactionState,
};


struct DependenciesInner {
    graph: Graph,
    stack: Stack,
    transaction_state: TransactionState,              //aktualny poziom tranzakcyjności
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            graph: Graph::new(),
            stack: Stack::new(),
            transaction_state: TransactionState::Idle,
        }
    }
}

#[derive(PartialEq)]
pub struct Dependencies {
    inner: Rc<EqBox<BoxRefCell<DependenciesInner>>>,
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
            inner: Rc::new(EqBox::new(BoxRefCell::new(DependenciesInner::new())))
        }
    }
}

impl Dependencies {
    pub fn new_value<T: PartialEq>(&self, value: T) -> Value<T> {
        Value::new(self.clone(), value)
    }

    pub fn new_computed_from<T: PartialEq>(&self, value: T) -> Computed<T> {
        let value = self.new_value(value);
        value.to_computed()
    }

    pub fn new_value_wrap_width_computed<T: PartialEq>(&self, value: T) -> Computed<Value<T>> {
        Value::new_value_wrap_width_computed(self.clone(), value)
    }

    pub fn transaction<F: FnOnce()>(&self, func: F) {
        let success = self.inner.value.change_no_params(|state| {
            state.transaction_state.up()
        });

        if !success {
            return;
        }

        func();

        let edges_values = self.inner.value.change_no_params(|state| {
            state.transaction_state.down()
        });

        if let Some(edges_values) = edges_values {
            let edges_to_refresh = self.inner.value.change(&edges_values, |state, edges_values| {
                state.graph.get_edges_to_refresh(edges_values)
            });

            refresh_edges::refresh_edges(&self, &edges_values, edges_to_refresh);

            self.inner.value.change_no_params(|state| {
                state.transaction_state.to_idle()
            });
        }
    }

    pub(crate) fn trigger_change(&self, parent_id: GraphId) {
        self.inner.value.change(parent_id, |state, parent_id| {
            state.transaction_state.add_edge_to_refresh(parent_id);
        });
    }

    pub(crate) fn add_graph_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.inner.value.change((parent_id, client_id), |state, (parent_id, client_id)| {
            state.graph.add_graph_connection(parent_id, client_id);
        });
    }

    pub(crate) fn remove_graph_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.inner.value.change((parent_id, client_id), |state, (parent_id, client_id)| {
            state.graph.remove_graph_connection(parent_id, client_id);
        });
    }

    pub(crate) fn refresh_token_add(&self, graph_value: GraphValueRefresh) {
        self.inner.value.change(graph_value, |state, graph_value| {
            state.graph.refresh_token_add(graph_value);
        });
    }

    pub(crate) fn refresh_token_drop(&self, id: GraphId) {
        self.inner.value.change(id, |state, id| {
            state.graph.refresh_token_drop(id);
        });
    }
    
    pub(crate) fn start_track(&self) {
        self.inner.value.change_no_params(|state| {
            state.stack.start_track();
        });
    }

    pub(crate) fn report_parent_in_stack(&self, parent_id: GraphId) {
        self.inner.value.change(parent_id, |state, parent_id| {
            state.stack.report_parent_in_stack(parent_id);
        });
    }

    pub(crate) fn stop_track(&self) -> BTreeSet<GraphId> {
        self.inner.value.change_no_params(|state| {
            state.stack.stop_track()
        })
    }

    pub(crate) fn get_parents(&self, client_id: GraphId) -> Vec<GraphId> {
        self.inner.value.get_with_context(client_id, |state, client_id| {
            state.graph.get_parents(client_id)
        })
    }

    pub fn from<T: PartialEq + 'static, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        let deps = self.clone();

        let get_value = Box::new(move || {
            let result = calculate();

            Rc::new(result)
        });

        Computed::new(deps, get_value)
    }

    pub fn all_connections_len(&self) -> u64 {
        self.inner.value.get(|state| {
            state.graph.all_connections_len()
        })
    }

    pub fn all_connections(&self) -> Vec<(GraphId, GraphId, u8)> {
        self.inner.value.get(|state| {
            state.graph.all_connections()
        })
    }
    
}
