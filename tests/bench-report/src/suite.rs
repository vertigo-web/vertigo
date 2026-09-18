//! What one benchmark suite declares about itself.
//!
//! Everything a suite needs to say in order to get a table, a file and a comparison is here,
//! as a `const`. Nothing in this crate knows what SSR or hydration are.

/// One timing column.
///
/// The unit is per metric rather than per suite because the reactive and DOM suites report a
/// batch in milliseconds and a per-operation cost in microseconds in the same row, and a noise
/// floor is meaningless without knowing which of the two it is expressed in.
pub struct Metric {
    /// JSON key and table heading, so the two cannot disagree.
    pub name: &'static str,
    pub unit: &'static str,
    /// Absolute noise floor, in `unit`.
    ///
    /// A delta has to clear this **and** the suite's relative band before it is printed as
    /// anything other than `~`. The percentage alone is not enough: a phase that takes three
    /// microseconds turns a one-microsecond scheduler hiccup into a 30% "regression", and two
    /// runs of the same commit reliably produce exactly that.
    pub floor: f64,
}

pub struct Suite {
    /// Written into the file and checked when one is read back, so a hydration baseline handed
    /// to the SSR suite is refused rather than silently producing an empty table.
    pub kind: &'static str,
    /// Directory under `target/bench/`.
    pub dir: &'static str,
    /// First line of the printed header.
    pub title: &'static str,
    /// Timing columns, in the order they should be read. For a suite whose metrics are phases,
    /// that is the order they happen in.
    pub metrics: &'static [Metric],
    /// The metric that gets the summary table and the stability check. Must name one of
    /// [`Self::metrics`].
    pub headline: &'static str,
    /// Counter columns worth tabulating, in order. A row may carry more counters than this -
    /// the DOM suite's per-variant command breakdown is per row rather than per suite - and
    /// every counter participates in change detection whether or not it has a column here.
    pub counters: &'static [&'static str],
    /// Below this, in relative terms, a delta is noise and prints as `~`.
    ///
    /// Deliberately conservative, and wider for the browser suites than the native one: a
    /// benchmark that cries wolf gets ignored.
    pub noise_band_pct: f64,
    /// `median / min` above this means something else was using the machine...
    pub instability_ratio: f64,
    /// ...but only once there is this much of it, in the headline metric's unit.
    pub instability_floor: f64,
    /// Environment variable naming a file to compare against.
    pub baseline_env: &'static str,
    /// Environment variable overriding the run label. Set when the two runs being compared are
    /// two implementations rather than two commits.
    pub label_env: &'static str,
}

impl Suite {
    pub(crate) fn metric(&self, name: &str) -> Option<&'static Metric> {
        self.metrics.iter().find(|metric| metric.name == name)
    }

    pub(crate) fn unit_of(&self, name: &str) -> &'static str {
        self.metric(name).map(|metric| metric.unit).unwrap_or("")
    }

    pub(crate) fn floor_of(&self, name: &str) -> f64 {
        self.metric(name).map(|metric| metric.floor).unwrap_or(0.0)
    }
}
