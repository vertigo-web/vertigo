//! Workload description and the measurement loop.

use std::cmp::Ordering;

use crate::clock::now_ms;

/// One benchmark, already constructed and primed.
pub trait Bench {
    /// Perform `iters` operations.
    ///
    /// The implementation keeps its own step counter and writes a different value every
    /// time: [`vertigo::Value::set`] short-circuits when the new value equals the old, so a
    /// workload that writes the same value twice would measure the equality check.
    fn run(&self, iters: u32);

    /// Compute-closure runs since the last call.
    ///
    /// This is the allocator-independent half of the measurement, and the only part worth
    /// asserting on - see the cutoff/fan-out invariants in `tests.rs`.
    fn take_runs(&self) -> u64;

    /// Folded from the subscribe callbacks and rendered into the DOM, so the data dependency
    /// runs from the computation all the way to a DOM write. Without it neither LLVM nor
    /// `wasm-opt -Os` has any reason to keep the work.
    fn checksum(&self) -> u64;
}

pub struct Workload {
    /// Stable identifier, used to build DOM ids and to key the report lines.
    pub slug: &'static str,
    pub title: &'static str,
    /// Operations per batch, tuned so a batch takes 100-300ms. See the baseline note in
    /// `workloads.rs`.
    pub iters: u32,
    pub make: fn() -> Box<dyn Bench>,
}

#[derive(Clone, PartialEq)]
pub struct Row {
    pub slug: &'static str,
    pub title: &'static str,
    pub iters: u32,
    pub best_ms: f64,
    pub median_ms: f64,
    pub runs: u64,
    pub checksum: u64,
}

impl Row {
    pub fn per_op_us(&self) -> f64 {
        self.best_ms * 1000.0 / f64::from(self.iters.max(1))
    }
}

const REPEATS: usize = 3;

/// Warm up, then take `REPEATS` batches and report the best and the median.
///
/// The best, not the mean: on a browser main thread the tail is GC and scheduler noise, so
/// the minimum is the closest thing to the real cost. The median is carried alongside so a
/// wildly unstable run is visible rather than hidden.
pub fn run_one(workload: &Workload, scale: f64) -> Row {
    let iters = ((f64::from(workload.iters) * scale) as u32).max(1);
    let bench = (workload.make)();

    // First-touch heap growth and the first `memory.grow` are not what we want to publish.
    bench.run((iters / 10).max(1));
    let _ = bench.take_runs();

    let mut samples: Vec<f64> = Vec::with_capacity(REPEATS);
    let mut runs = 0;

    for _ in 0..REPEATS {
        let start = now_ms();
        bench.run(iters);
        samples.push(now_ms() - start);
        runs = bench.take_runs();
    }

    samples.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    Row {
        slug: workload.slug,
        title: workload.title,
        iters,
        best_ms: samples.first().copied().unwrap_or(0.0),
        median_ms: samples.get(REPEATS / 2).copied().unwrap_or(0.0),
        runs,
        checksum: bench.checksum(),
    }
    // `bench` drops here, before the next workload builds its graph, so workloads never
    // overlap in the heap.
}

/// One line per workload, `|`-separated: the test parses this instead of hunting for
/// per-workload element ids, so adding a workload does not touch the test.
pub fn report_text(rows: &[Row]) -> String {
    let mut out = String::new();
    for row in rows {
        out.push_str(&format!(
            "{}|{}|{:.3}|{:.3}|{:.4}|{}|{}\n",
            row.slug,
            row.iters,
            row.best_ms,
            row.median_ms,
            row.per_op_us(),
            row.runs,
            row.checksum,
        ));
    }
    out
}
