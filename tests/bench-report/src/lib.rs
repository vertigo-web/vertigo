//! Turning benchmark samples into a table, a JSON file, and a comparison against an earlier
//! run - for all four suites at once.
//!
//! `tests/ssr-bench/` and `tests/hydration-bench/` each used to carry a near-identical copy of
//! this. The duplication was deliberate while the hydration suite had to cherry-pick onto a
//! branch that did not have the SSR one; now that every suite lives on the same branch, the
//! copies have been replaced by this crate, and the reactive-graph and DOM suites - which had
//! no persistence at all - are on the same footing.
//!
//! ## The model
//!
//! A run is a list of [`Row`]s. A row has a key, some [`Stats`] per named [`Metric`], and some
//! integer counters. That is the whole vocabulary, and it is enough for all four suites:
//!
//! | suite | key | metrics | counters |
//! |---|---|---|---|
//! | ssr | subject/route | total and eight phases, us | commands, bytes, host and guest calls |
//! | hydration | subject/route | boot, render, hydrate, total, ms | coverage, DOM mutations |
//! | reactive | workload | batch ms, per-op us | iterations, compute runs, checksum |
//! | dom | workload | batch ms, per-op us | the above, plus DOM commands by variant |
//!
//! ## What is compared, and what is only printed
//!
//! Metrics are timings and are printed with a delta that has to clear both a relative band and
//! an absolute floor before it is shown as anything other than `~`. Counters are deterministic:
//! a change in one is reported separately and **before** the timings, because a row that got
//! slower may simply have been asked to do more.

mod artifact;
mod browser;
mod compare;
mod meta;
mod run;
mod stats;
mod suite;

#[cfg(test)]
mod tests;

pub use artifact::Artifact;
pub use browser::{parse_samples, scale};
pub use compare::Baseline;
pub use meta::{Meta, body_hash, now_unix};
pub use run::{Row, Run};
pub use stats::{Stats, micros};
pub use suite::{Metric, Suite};

/// Same type as `fantoccini_tests::TestResult`, spelled out here so this crate does not have
/// to depend on the test harness that uses it.
pub type BenchResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// The schema version written into every file.
///
/// Bumped from 1 when the four suites were unified onto one shape. Files written before that
/// are still readable - see [`Baseline`] - so an old baseline is compared rather than refused.
pub const SCHEMA: u64 = 2;

/// Attaches a description of the step to a failure, so a propagated error says which line
/// produced it rather than only what went wrong.
pub(crate) trait Ctx<T> {
    fn ctx(self, what: impl std::fmt::Display) -> BenchResult<T>;
}

impl<T, E: std::fmt::Display> Ctx<T> for Result<T, E> {
    fn ctx(self, what: impl std::fmt::Display) -> BenchResult<T> {
        self.map_err(|err| format!("{what}: {err}").into())
    }
}
