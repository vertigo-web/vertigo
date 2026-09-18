//! Reducing a vector of samples to the three numbers worth carrying around.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub min: f64,
    pub median: f64,
    pub p90: f64,
}

impl Stats {
    /// Consumes the samples, because it sorts them.
    ///
    /// `min` is the headline: on a machine with a scheduler, a page cache and - in the browser
    /// suites - a garbage collector, the tail is interference, and the fastest observation is
    /// the closest thing to the real cost. `median` rides alongside so that an unstable run is
    /// visible rather than hidden, and `p90` because it is free and it is what moves when a
    /// change affects the tail rather than the typical case.
    ///
    /// With very few samples `p90` degenerates to the maximum. That is not a defect to correct
    /// here; it is a property of the suite that took three samples, and it is the reason the
    /// suites that can afford ten take ten.
    pub fn of(mut samples: Vec<f64>) -> Stats {
        if samples.is_empty() {
            return Stats::default();
        }

        samples.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));

        let last = samples.len() - 1;
        let at = |index: usize| samples.get(index.min(last)).copied().unwrap_or(0.0);

        Stats {
            min: at(0),
            median: at(samples.len() / 2),
            p90: at(samples.len() * 9 / 10),
        }
    }

    /// A run this uneven was sharing the machine with something.
    ///
    /// Both bars have to be cleared. The ratio alone flags every small number: a row whose
    /// total is two milliseconds crosses a 1.5 ratio on a third of a millisecond of scheduler
    /// jitter, which is not a busy machine, it is a small number.
    pub(crate) fn unstable(&self, ratio: f64, floor: f64) -> bool {
        self.min > 0.0 && self.median - self.min > floor && self.median / self.min > ratio
    }
}

/// Microseconds, as `f64`, so that what the table prints and what the file stores are the same
/// number and a delta is a subtraction.
pub fn micros(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.0
}
