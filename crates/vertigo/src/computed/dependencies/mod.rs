use std::{
    rc::Rc,
};

use crate::{
    computed::{GraphId, GraphValueRefresh}, DropResource, Context,
};

use super::graph_id::GraphIdKind;


mod external_connections;
mod graph;
pub mod hook;
mod refresh;
mod stack;
mod transaction_state;
mod graph_connections;

use {
    graph::Graph,
    transaction_state::TransactionState,
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
    transaction_state: Rc<TransactionState>,
}

impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Dependencies {
            graph: self.graph.clone(),
            transaction_state: self.transaction_state.clone(),
        }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Self {
            graph: Rc::new(
                Graph::new(),
            ),
            transaction_state: Rc::new(
                TransactionState::new()
            ),
        }
    }
}

impl Dependencies {
    pub fn set_hook(&self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.transaction_state.set_hook(before_start, after_end);
    }

    pub fn transaction<F: FnOnce(&Context)>(&self, func: F) {
        let success = self.transaction_state.up();

        if !success {
            return;
        }

        self.graph.stack.ignore_tracking_on();
        let context = Context::new();
        func(&context);
        self.graph.stack.ignore_tracking_off();

        let edges_values = self.transaction_state.down();

        if let Some(edges_values) = edges_values {
            let mut edges_client = Vec::new();

            for id in self.graph.get_all_deps(edges_values) {
                match id.get_type() {
                    GraphIdKind::Value => {
                        unreachable!();
                    },
                    GraphIdKind::Computed => {
                        self.graph.refresh.clear_cache(&id);
                    },
                    GraphIdKind::Client => {
                        edges_client.push(id);
                    }
                }
            }

            for id in edges_client {
                self.graph.refresh.refresh(&id);
            }

            self.graph.recalculate_edges();
            self.transaction_state.move_to_idle();
        }
    }

    pub(crate) fn trigger_change(&self, parent_id: GraphId) {
        self.transaction_state.add_edge_to_refresh(parent_id);
    }

    pub(crate) fn refresh_token_add(&self, graph_value: GraphValueRefresh) {
        self.graph.refresh.refresh_token_add(graph_value);
    }

    pub(crate) fn refresh_token_drop(&self, id: GraphId) {
        self.graph.refresh.refresh_token_drop(id);
    }

    pub(crate) fn start_track(&self, client_id: GraphId) {
        self.graph.stack.start_track(client_id);
    }

    pub(crate) fn report_parent_in_stack(&self, parent_id: GraphId) {
        self.graph.stack.report_parent_in_stack(parent_id);
    }

    pub(crate) fn stop_track(&self) {
        self.graph.stack.stop_track()
    }

    pub(crate) fn block_tracking_on(&self) {
        self.graph.stack.block_tracking_on()
    }

    pub(crate) fn block_tracking_off(&self) {
        self.graph.stack.block_tracking_off()
    }

    pub(crate) fn remove_client(&self, client_id: GraphId) {
        self.graph.remove_client(client_id);
    }

    pub fn all_connections_len(&self) -> u64 {
        self.graph.all_connections_len()
    }

    pub fn external_connections_register_connect(&self, id: GraphId, connect: Rc<dyn Fn() -> DropResource>) {
        self.graph.external_connections.register_connect(id, connect);
    }

    pub fn external_connections_unregister_connect(&self, id: GraphId) {
        self.graph.external_connections.unregister_connect(id);
    }

    pub fn external_connections_refresh(&self) {
        if self.transaction_state.is_idle() {
            self.graph.recalculate_edges();
        }
    }
}
