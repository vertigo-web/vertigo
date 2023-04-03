use std::{collections::BTreeSet};

use crate::{Context, GraphId};

use self::hook::Hooks;

use super::graph_id::GraphIdKind;


mod external_connections;
mod graph;
pub mod hook;
mod refresh;
mod transaction_state;
mod graph_connections;
mod graph_one_to_many;

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
/// - Render function (a component) takes a computed state provided by the graph and returns a rendered element ([DomElement](struct.DomElement.html)).
/// - Upon change in VDOM the real DOM is also updated.
/// - Components can provide the DOM with functions that get fired on events like [on_click](struct.DomElement.html#structfield.on_click), which may modify the state, thus triggering necessary computing once again.
pub struct Dependencies {
    pub(crate) graph: Graph,
    transaction_state: TransactionState,
    pub(crate) hooks: Hooks,
}

impl Default for Dependencies {
    fn default() -> Self {
        Self {
            graph: Graph::new(),
            transaction_state: TransactionState::new(),
            hooks: Hooks::new(),
        }
    }
}

impl Dependencies {
    pub fn transaction<R, F: FnOnce(&Context) -> R>(&self, func: F) -> R {
        self.transaction_state.up();

        let context = Context::new();
        let result = func(&context);
        let _ = context;

        let edges_values = self.transaction_state.down();

        let Some(client_ids) = edges_values else {
            return result;
        };

        for id in client_ids {
            self.graph.refresh.refresh(&id);
        }

        self.transaction_state.move_to_idle();
        self.hooks.fire_end();

        result
    }

    pub(crate) fn report_set(&self, value_id: GraphId) {
        let mut client = BTreeSet::new();

        for id in self.graph.connections.get_all_deps(value_id) {
            match id.get_type() {
                GraphIdKind::Value => {
                    unreachable!();
                },
                GraphIdKind::Computed => {
                    self.graph.refresh.clear_cache(&id);
                },
                GraphIdKind::Client => {
                    client.insert(id);
                }
            }
        }

        self.transaction_state.add_clients_to_refresh(client);
    }
}
