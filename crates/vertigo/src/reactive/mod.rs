//! Transactional reactive graph with equality cutoff.
//!
//! After a transaction, dirty nodes are processed in topological order (a node is
//! ready when none of its parents are dirty). `get` during the wave pulls a stale
//! parent before returning its cache. After a node becomes clean, dependents are
//! enqueued **only if** the node's value changed ([`PartialEq`]).
//!
//! Domain invariants for this bounded context: [`invariants`].

mod computed;
mod context;
mod drop_resource;
mod graph;
#[doc = include_str!("invariants.md")]
pub mod invariants {}
mod to_computed;
mod value;

pub use computed::Computed;
pub use context::Context;
pub use drop_resource::DropResource;
pub use graph::{Graph, GraphId};
pub use to_computed::ToComputed;
pub use value::Value;

/// Types that behave like a [`Value<T>`].
pub trait Reactive<T>: PartialEq {
    fn set(&self, value: T);
    fn get(&self, context: &Context) -> T;
    fn change(&self, change_fn: impl FnOnce(&mut T));
}

impl<T> Reactive<T> for Value<T>
where
    T: Clone + PartialEq + 'static,
{
    fn set(&self, value: T) {
        Value::set(self, value)
    }

    fn get(&self, context: &Context) -> T {
        Value::get(self, context)
    }

    fn change(&self, change_fn: impl FnOnce(&mut T)) {
        Value::change(self, change_fn)
    }
}

thread_local! {
    static DEFAULT_GRAPH: Graph = Graph::new();
}

pub(crate) fn default_graph() -> Graph {
    DEFAULT_GRAPH.with(Graph::clone)
}

/// Run `f` as a transaction on the default graph.
///
/// Nested calls are allowed. Propagation runs when the outermost transaction ends.
pub fn transaction<R>(f: impl FnOnce(&Context) -> R) -> R {
    default_graph().transaction(f)
}

/// Register a callback on the default graph, fired after each completed transaction
/// (including a lone [`Value::set`]).
pub fn on_after_transaction(callback: impl Fn() + 'static) -> DropResource {
    default_graph().on_after_transaction(callback)
}

#[cfg(test)]
mod propagation_order;
#[cfg(test)]
mod tests;
