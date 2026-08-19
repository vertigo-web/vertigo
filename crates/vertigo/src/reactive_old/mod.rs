//! Previous reactive graph, compiled only in tests for comparison with [`crate::reactive`].
#![allow(dead_code)]

mod computed;
mod context;
mod dependencies;
mod graph_id;
mod graph_value;
mod reactive;
mod to_computed;
mod value;
mod value_inner;

pub(crate) use computed::Computed;
pub(crate) use context::Context;
pub(crate) use dependencies::{Dependencies, get_dependencies};
pub(crate) use graph_id::GraphId;
pub(crate) use graph_value::GraphValue;
pub(crate) use to_computed::ToComputed;
pub(crate) use value::Value;

pub(crate) fn transaction<R>(f: impl FnOnce(&Context) -> R) -> R {
    get_dependencies().transaction(f)
}

mod compare;
