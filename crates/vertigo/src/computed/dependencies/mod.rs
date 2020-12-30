use std::rc::Rc;
use std::cmp::PartialEq;
use std::collections::BTreeSet;

use crate::computed::{
    Value,
    Computed,
    GraphId,
    GraphValueRefresh,
};
use crate::utils::{
    BoxRefCell,
    EqBox,
};

mod graph;
mod graph_map;
mod transaction_state;
mod stack;
mod refresh;
pub mod refresh_edges;

use {
    graph::Graph,
    stack::Stack,
    transaction_state::TransactionState,
    refresh::Refresh,
};

#[derive(PartialEq)]
pub struct Dependencies {
    graph: Rc<EqBox<BoxRefCell<Graph>>>,
    stack: Rc<EqBox<BoxRefCell<Stack>>>,
    refresh: Rc<EqBox<BoxRefCell<Refresh>>>,
    transaction_state: Rc<EqBox<BoxRefCell<TransactionState>>>,
}

impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Dependencies {
            graph: self.graph.clone(),
            stack: self.stack.clone(),
            refresh: self.refresh.clone(),
            transaction_state: self.transaction_state.clone(),
        }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Self {
            graph: Rc::new(EqBox::new(BoxRefCell::new(Graph::new()))),
            stack: Rc::new(EqBox::new(BoxRefCell::new(Stack::new()))),
            refresh: Rc::new(EqBox::new(BoxRefCell::new(Refresh::new()))),
            transaction_state: Rc::new(EqBox::new(BoxRefCell::new(TransactionState::Idle))),        }
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
        let success = self.transaction_state.value.change_no_params(|state| {
            state.up()
        });

        if !success {
            return;
        }

        func();

        let edges_values = self.transaction_state.value.change_no_params(|state| {
            state.down()
        });

        if let Some(edges_values) = edges_values {
            let edges_to_refresh = self.get_edges_to_refresh(&edges_values);

            refresh_edges::refresh_edges(&self, &edges_values, edges_to_refresh);

            self.transaction_state.value.change_no_params(|state| {
                state.to_idle()
            });
        }
    }

    fn get_edges_to_refresh(&self, edges: &BTreeSet<GraphId>) -> Vec<GraphValueRefresh> {

        let mut result = Vec::new();

        for id in self.get_all_deps(edges) {
            if let Some(item) = self.refresh_get(&id) {
                result.push(item);
            } else {
                log::error!("Missing refresh token for(1) {:?}", id);
            }
        }

        result
    }

    fn get_all_deps(&self, edges: &BTreeSet<GraphId>) -> BTreeSet<GraphId> {
        self.graph.value.get_with_context(edges, |state, edges| {
            state.get_all_deps(edges)
        })
    }

    pub(crate) fn trigger_change(&self, parent_id: GraphId) {
        self.transaction_state.value.change(parent_id, |state, parent_id| {
            state.add_edge_to_refresh(parent_id);
        });
    }

    pub(crate) fn add_graph_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.graph.value.change((parent_id, client_id), |state, (parent_id, client_id)| {
            state.add_graph_connection(parent_id, client_id);
        });
    }

    pub(crate) fn remove_graph_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.graph.value.change((parent_id, client_id), |state, (parent_id, client_id)| {
            state.remove_graph_connection(parent_id, client_id);
        });
    }

    pub(crate) fn refresh_token_add(&self, graph_value: GraphValueRefresh) {
        self.refresh.value.change(graph_value, |state, graph_value| {
            state.refresh_token_add(graph_value);
        });
    }

    pub(crate) fn refresh_token_drop(&self, id: GraphId) {
        self.refresh.value.change(id, |state, id| {
            state.refresh_token_drop(id);
        });
    }

    pub(crate) fn start_track(&self) {
        self.stack.value.change_no_params(|state| {
            state.start_track();
        });
    }

    pub(crate) fn report_parent_in_stack(&self, parent_id: GraphId) {
        self.stack.value.change(parent_id, |state, parent_id| {
            state.report_parent_in_stack(parent_id);
        });
    }

    pub(crate) fn stop_track(&self) -> BTreeSet<GraphId> {
        self.stack.value.change_no_params(|state| {
            state.stop_track()
        })
    }

    pub(crate) fn get_parents(&self, client_id: GraphId) -> Vec<GraphId> {
        self.graph.value.get_with_context(client_id, |state, client_id| {
            state.get_parents(client_id)
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
        self.graph.value.get(|state| {
            state.all_connections_len()
        })
    }

    pub fn all_connections(&self) -> Vec<(GraphId, GraphId, u8)> {
        self.graph.value.get(|state| {
            state.all_connections()
        })
    }

    pub fn has_listeners(&self, parent_id: &GraphId) -> bool {
        self.graph.value.get_with_context(parent_id, |state, parent_id| {
            state.has_listeners(parent_id)
        })
    }

    pub fn refresh_get(&self, id: &GraphId) -> Option<GraphValueRefresh> {
        self.refresh.value.get_with_context(id, |state, id| {
            state.get(id)
        })
    }

    pub fn drop_value(&self, parent_id: &GraphId) {
        self.refresh.value.get_with_context(parent_id, |state, parent_id| {
            state.drop_value(parent_id);
        })
    }

    pub fn drain_removables(&self) -> Vec<GraphId> {
        self.graph.value.change_no_params(|state| {
            state.drain_removables()
        })
    }
}
