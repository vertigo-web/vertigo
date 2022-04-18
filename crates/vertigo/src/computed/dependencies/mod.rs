use std::{
    collections::BTreeSet,
    rc::Rc,
};

use crate::{
    computed::{Computed, GraphId, GraphValueRefresh, Value}, DropResource,
};

use super::value::ToRc;

mod external_connections;
mod graph;
mod graph_map;
pub mod hook;
mod refresh;
mod stack;
mod transaction_state;

use {
    external_connections::ExternalConnections,
    graph::Graph,
    stack::Stack,
    transaction_state::TransactionState,
    refresh::Refresh,
};

/// A graph of values and clients that can automatically compute what to refresh after one value change.
///
/// A [Driver](struct.Driver.html) object wrapps dependency graph, so you do not need to use this under normal circumstances.
///
/// - Dependency graph holds values, computed values ([computeds](struct.Computed.html)) and clients (render functions).
/// - Upon changing some value all dependent computeds get computed, and all dependent clients get rendered.
/// - Render function (a component) takes a computed state provided by the graph and returns a rendered element ([VDomElement](struct.VDomElement.html)).
/// - Upon change in VDOM the real DOM is also updated.
/// - Components can provide the DOM with functions that get fired on events like [on_click](struct.VDomElement.html#structfield.on_click), which may modify the state, thus triggering necessary computing once again.
pub struct Dependencies {
    graph: Rc<Graph>,
    stack: Rc<Stack>,
    refresh: Rc<Refresh>,
    transaction_state: Rc<TransactionState>,
    external_connections: ExternalConnections,
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
        let refresh: Refresh = Refresh::new();

        Self {
            graph: Rc::new(
                Graph::new(external_connections.clone(), refresh.clone()),
            ),
            stack: Rc::new(Stack::new()),
            refresh: Rc::new(refresh),
            transaction_state: Rc::new(
                TransactionState::new()
            ),
            external_connections,
        }
    }
}

impl Dependencies {
    pub fn new_value<T>(&self, value: impl ToRc<T>) -> Value<T> {
        Value::new(self.clone(), value)
    }

    pub fn new_with_connect<T, F>(&self, value: T, create: F) -> Computed<T>
    where
        F: Fn(&Value<T>) -> DropResource + 'static
    {
        Value::<T>::new_selfcomputed_value::<F>(self.clone(), value, create)
    }

    pub fn new_computed_from<T>(&self, value: impl ToRc<T>) -> Computed<T> {
        let value = self.new_value(value);
        value.to_computed()
    }

    pub fn set_hook(&self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.transaction_state.set_hook(before_start, after_end);
    }

    pub fn transaction<F: FnOnce()>(&self, func: F) {
        let success = self.transaction_state.up();

        if !success {
            return;
        }

        func();

        let edges_values = self.transaction_state.down();

        if let Some(edges_values) = edges_values {
            let edges_to_refresh = self.get_edges_to_refresh(edges_values);

            let mut edges_client = Vec::new();

            for item in edges_to_refresh {
                if item.is_computed() {
                    item.drop_value();
                } else {
                    edges_client.push(item);
                }
            }
        
            for item in edges_client {
                item.refresh();
            }
        
            self.external_connections.refresh_connect();

            self.transaction_state.move_to_idle();
        }
    }

    fn get_edges_to_refresh(&self, edges: BTreeSet<GraphId>) -> Vec<GraphValueRefresh> {
        let mut result = Vec::new();

        for id in self.graph.get_all_deps(edges) {
            if let Some(item) = self.refresh.get(&id) {
                result.push(item);
            } else {
                log::error!("Missing refresh token for(1) {:?}", id);
            }
        }

        result
    }

    pub(crate) fn trigger_change(&self, parent_id: GraphId) {
        self.transaction_state.add_edge_to_refresh(parent_id);
    }

    pub(crate) fn set_parent_for_client(&self, client_id: GraphId, parents_list: BTreeSet<GraphId>) {
        self.graph.set_parent_for_client(client_id, parents_list);
    }

    pub(crate) fn remove_client(&self, client_id: GraphId) {
        self.graph.remove_client(client_id);
    }

    pub(crate) fn refresh_token_add(&self, graph_value: GraphValueRefresh) {
        self.refresh.refresh_token_add(graph_value);
    }

    pub(crate) fn refresh_token_drop(&self, id: GraphId) {
        self.refresh.refresh_token_drop(id);
    }

    pub(crate) fn start_track(&self) {
        self.stack.start_track();
    }

    pub(crate) fn report_parent_in_stack(&self, parent_id: GraphId) {
        self.stack.report_parent_in_stack(parent_id);
    }

    pub(crate) fn stop_track(&self) -> BTreeSet<GraphId> {
        self.stack.stop_track()
    }

    pub fn from<T: 'static, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        let deps = self.clone();
        Computed::new(deps, move || Rc::new(calculate()))
    }

    pub fn all_connections_len(&self) -> u64 {
        self.graph.all_connections_len()
    }

    pub fn external_connections_register_connect(&self, id: GraphId, connect: Rc<dyn Fn() -> DropResource>) {
        self.external_connections.register_connect(id, connect);
    }

    pub fn external_connections_unregister_connect(&self, id: GraphId) {
        self.external_connections.unregister_connect(id);
    }

    pub fn external_connections_refresh(&self) {
        if self.transaction_state.is_idle() {
            self.external_connections.refresh_connect();
        }
    }
}
