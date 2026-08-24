//! Counting the DOM commands one operation emits.
//!
//! This is the DOM analogue of the compute-run counters that guard the reactive-graph
//! suite: deterministic, machine-independent, and therefore the thing worth asserting on.
//! A wall-clock figure says a change got slower; a command count says what it now does
//! differently.

use vertigo::{
    JsJson,
    dev::{
        command::DriverDomCommand,
        inspect::{log_start, log_take},
    },
    get_driver,
};

use crate::runner::Workload;

pub const VARIANT_NAMES: [&str; 13] = [
    "CreateNode",
    "CreateText",
    "UpdateText",
    "SetAttr",
    "RemoveAttr",
    "RemoveNode",
    "RemoveText",
    "InsertBefore",
    "InsertCss",
    "CreateComment",
    "RemoveComment",
    "CallbackAdd",
    "CallbackRemove",
];

#[derive(Clone, PartialEq, Eq, Default)]
pub struct CmdCounts {
    pub total: u32,
    pub by_variant: [u32; VARIANT_NAMES.len()],
}

impl CmdCounts {
    /// `Name=count` pairs for the non-zero variants, or `-` when the operation emitted
    /// nothing. Never the empty string: the report is `|`-separated and a field that can
    /// vanish would make the field count depend on the data.
    pub fn breakdown(&self) -> String {
        let parts: Vec<String> = self
            .by_variant
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(index, count)| format!("{}={count}", VARIANT_NAMES[index]))
            .collect();

        if parts.is_empty() {
            "-".to_string()
        } else {
            parts.join(",")
        }
    }
}

fn variant_index(command: &DriverDomCommand) -> usize {
    match command {
        DriverDomCommand::CreateNode { .. } => 0,
        DriverDomCommand::CreateText { .. } => 1,
        DriverDomCommand::UpdateText { .. } => 2,
        DriverDomCommand::SetAttr { .. } => 3,
        DriverDomCommand::RemoveAttr { .. } => 4,
        DriverDomCommand::RemoveNode { .. } => 5,
        DriverDomCommand::RemoveText { .. } => 6,
        DriverDomCommand::InsertBefore { .. } => 7,
        DriverDomCommand::InsertCss { .. } => 8,
        DriverDomCommand::CreateComment { .. } => 9,
        DriverDomCommand::RemoveComment { .. } => 10,
        DriverDomCommand::CallbackAdd { .. } => 11,
        DriverDomCommand::CallbackRemove { .. } => 12,
    }
}

/// Count the DOM commands one operation emits, on a scene of its own.
///
/// Separate from the timed pass for two reasons. The inspect tap clones every command it
/// sees, so a tapped run is not a timed run. And the count wants to be exact rather than
/// averaged: one operation on a primed scene is deterministic.
pub fn count_one(workload: &Workload) -> CmdCounts {
    let bench = (workload.make)();

    // One untapped operation first. Everything that happens once and never again - a css
    // registration, the first value a `class` attribute is ever given, a lazily built
    // `Computed`'s first evaluation - lands here, so what the tap sees below is the
    // steady-state cost of an operation rather than its first-time cost.
    bench.run(1);

    log_start();
    bench.run(1);
    let commands = log_take();

    let mut counts = CmdCounts::default();
    for command in &commands {
        counts.total += 1;
        counts.by_variant[variant_index(command)] += 1;
    }
    counts
    // `bench` drops here, so the counting scene leaves the stage before the next workload
    // builds its own.
}

/// How many DOM nodes vertigo is currently tracking, read out of the JS-side node map.
///
/// Taken with the stage empty before and after a workload, the difference is scene state
/// that failed to tear down - which would otherwise show up only as an unexplained
/// slowdown in some later workload.
///
/// The `js!` macro cannot express this: it parses its argument as a Rust expression and
/// `$vertigoApi` is not an identifier. This is the chain `js!` would have built.
pub fn live_node_count() -> u64 {
    let json = get_driver()
        .dom_access()
        .root("window")
        .get("$vertigoApi")
        .get("dom")
        .get("nodes")
        .get("data")
        .get("size")
        .fetch();

    match json {
        JsJson::Number(number) => number.as_f64() as u64,
        _ => 0,
    }
}
