use std::rc::Rc;
use std::cmp::PartialEq;
use std::collections::BTreeSet;
use std::any::Any;

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

use self::external_connections::ExternalConnections;

use super::value::ToRc;

mod graph;
mod graph_map;
mod transaction_state;
mod stack;
mod refresh;
pub mod refresh_edges;
pub mod hook;
mod external_connections;

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
    pub external_connections: ExternalConnections,
}

impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Dependencies {
            graph: self.graph.clone(),
            stack: self.stack.clone(),
            refresh: self.refresh.clone(),
            transaction_state: self.transaction_state.clone(),
            external_connections: self.external_connections.clone(),
        }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        let external_connections = ExternalConnections::default();

        Self {
            graph: Rc::new(EqBox::new(BoxRefCell::new(
                Graph::new(external_connections.clone()),
                "graph"
            ))),
            stack: Rc::new(EqBox::new(BoxRefCell::new(Stack::new(), "stack"))),
            refresh: Rc::new(EqBox::new(BoxRefCell::new(Refresh::new(), "refresh"))),
            transaction_state: Rc::new(EqBox::new(BoxRefCell::new(TransactionState::new(), "transaction_state"))),
            external_connections,
        }
    }
}

impl Dependencies {
    pub fn new_value<T: PartialEq>(&self, value: impl ToRc<T>) -> Value<T> {
        Value::new(self.clone(), value)
    }

    pub fn new_with_connect<T: PartialEq, F: Fn(&Value<T>) -> Box<dyn Any> + 'static>(&self, value: T, create: F) -> Computed<T> {
        Value::<T>::new_selfcomputed_value::<F>(self.clone(), value, create)
    }


    pub fn new_computed_from<T: PartialEq>(&self, value: impl ToRc<T>) -> Computed<T> {
        let value = self.new_value(value);
        value.to_computed()
    }

    pub fn set_hook(&self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.transaction_state.change(
            (before_start, after_end), 
            |state, (before_start, after_end)| {
                state.set_hook(before_start, after_end);
            }
        );
    }

    pub fn transaction<F: FnOnce()>(&self, func: F) {
        let success = self.transaction_state.change((), |state, _| {
            state.up()
        });

        if !success {
            return;
        }

        func();

        let edges_values = self.transaction_state.change((), |state, _| {
            state.down()
        });

        if let Some(edges_values) = edges_values {
            let edges_to_refresh = self.get_edges_to_refresh(&edges_values);

            refresh_edges::refresh_edges(self, &edges_values, edges_to_refresh);

            self.transaction_state.change((), |state, _| {
                state.move_to_idle()
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
        self.graph.get_with_context(edges, |state, edges| {
            state.get_all_deps(edges)
        })
    }

    pub(crate) fn trigger_change(&self, parent_id: GraphId) {
        self.transaction_state.change(parent_id, |state, parent_id| {
            state.add_edge_to_refresh(parent_id);
        });
    }

    pub(crate) fn add_graph_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.graph.change((parent_id, client_id), |state, (parent_id, client_id)| {
            state.add_graph_connection(parent_id, client_id);
        });
    }

    pub(crate) fn remove_graph_connection(&self, parent_id: GraphId, client_id: GraphId) {
        self.graph.change((parent_id, client_id), |state, (parent_id, client_id)| {
            state.remove_graph_connection(parent_id, client_id)
        });
    }

    pub(crate) fn refresh_token_add(&self, graph_value: GraphValueRefresh) {
        self.refresh.change(graph_value, |state, graph_value| {
            state.refresh_token_add(graph_value);
        });
    }

    pub(crate) fn refresh_token_drop(&self, id: GraphId) {
        self.refresh.change(id, |state, id| {
            state.refresh_token_drop(id);
        });
    }

    pub(crate) fn start_track(&self) {
        self.stack.change((), |state, _| {
            state.start_track();
        });
    }

    pub(crate) fn report_parent_in_stack(&self, parent_id: GraphId) {
        self.stack.change(parent_id, |state, parent_id| {
            state.report_parent_in_stack(parent_id);
        });
    }

    pub(crate) fn stop_track(&self) -> BTreeSet<GraphId> {
        self.stack.change((), |state, _| {
            state.stop_track()
        })
    }

    pub(crate) fn get_parents(&self, client_id: GraphId) -> Vec<GraphId> {
        self.graph.get_with_context(client_id, |state, client_id| {
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
        self.graph.get(|state| {
            state.all_connections_len()
        })
    }

    pub fn all_connections(&self) -> Vec<(GraphId, GraphId, u8)> {
        self.graph.get(|state| {
            state.all_connections()
        })
    }

    pub fn has_listeners(&self, parent_id: &GraphId) -> bool {
        self.graph.get_with_context(parent_id, |state, parent_id| {
            state.has_listeners(parent_id)
        })
    }

    pub fn refresh_get(&self, id: &GraphId) -> Option<GraphValueRefresh> {
        self.refresh.get_with_context(id, |state, id| {
            state.get(id)
        })
    }

    pub fn drop_value(&self, parent_id: &GraphId) {
        self.refresh.get_with_context(parent_id, |state, parent_id| {
            state.drop_value(parent_id);
        })
    }

    pub fn drain_removables(&self) -> Vec<GraphId> {
        self.graph.change((), |state, ()| {
            state.drain_removables()
        })
    }
}
