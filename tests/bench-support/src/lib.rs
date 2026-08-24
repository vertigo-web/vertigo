//! Shared scaffolding for the browser benchmark suites.
//!
//! Two apps use this: `tests/reactive-bench` (the reactive graph, measured with the DOM
//! deliberately out of the picture) and `tests/dom-bench` (real rendering and DOM
//! operations). They share the clock, the batching loop and - crucially - the report line
//! format, which the fantoccini side parses. Duplicating that format is how the two suites
//! would quietly drift apart.
//!
//! The DOM-specific measurements ([`counts`], and the node census) are opt-in through
//! [`runner::RunOpts`], so a suite that emits no DOM commands pays nothing for them and its
//! report line stays exactly as it was.

pub mod clock;
pub mod counts;
pub mod runner;

pub use clock::{now_ms, read_scale, user_agent};
pub use counts::{CmdCounts, VARIANT_NAMES};
pub use runner::{Bench, Row, RunOpts, Workload, report_text, run_one};
