//! Workload description and the measurement loop.

use std::cmp::Ordering;

use crate::{
    clock::now_ms,
    counts::{CmdCounts, count_one, live_node_count},
};

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
    /// Every batch time, in the order they were taken.
    ///
    /// `best_ms` and `median_ms` are what this app renders into its own table; the raw samples
    /// go out as well so the driver can reduce them the same way the other suites reduce
    /// theirs, and so a run's spread survives into the JSON rather than being thrown away here.
    ///
    /// Arrival order rather than sorted, which is the more informative of the two and the only
    /// one that cannot be recovered afterwards: it is what says whether the first batch is
    /// systematically the slow one, i.e. whether the warm-up above is doing its job. Everything
    /// that wants them ordered sorts its own copy.
    pub samples_ms: Vec<f64>,
    pub runs: u64,
    pub checksum: u64,
    /// `Some` only when [`RunOpts::count_commands`] is set. When it is `None` the report
    /// line keeps its original seven fields, which is what lets the reactive-graph suite
    /// share this runner without its test file changing.
    pub cmds: Option<CmdCounts>,
    /// Tracked DOM nodes left behind by the workload, measured with the stage empty on
    /// both sides. Only meaningful when [`RunOpts::node_census`] is set.
    pub nodes_leaked: i64,
}

impl Row {
    pub fn per_op_us(&self) -> f64 {
        self.best_ms * 1000.0 / f64::from(self.iters.max(1))
    }
}

/// Extra measurements a suite can ask for. Both default to off, so a suite that emits no
/// DOM commands never installs the inspect tap and never pays for the census.
#[derive(Clone, Copy, Default)]
pub struct RunOpts {
    pub count_commands: bool,
    pub node_census: bool,
}

const REPEATS: usize = 3;

/// Warm up, then take `REPEATS` batches and report the best and the median.
///
/// The best, not the mean: on a browser main thread the tail is GC and scheduler noise, so
/// the minimum is the closest thing to the real cost. The median is carried alongside so a
/// wildly unstable run is visible rather than hidden.
pub fn run_one(workload: &Workload, scale: f64, opts: RunOpts) -> Row {
    // Both censuses are taken with the stage empty: `run_timed` drops its bench before
    // returning and `count_one` drops its own, and no rendering happens in between.
    let nodes_before = opts.node_census.then(live_node_count);

    let mut row = run_timed(workload, scale);

    if opts.count_commands {
        row.cmds = Some(count_one(workload));
    }
    if let Some(before) = nodes_before {
        row.nodes_leaked = live_node_count() as i64 - before as i64;
    }
    row
}

fn run_timed(workload: &Workload, scale: f64) -> Row {
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

    // A copy, so `samples` keeps the order the batches were taken in - which is what
    // `Row::samples_ms` publishes and what a reader needs to see a warm-up effect.
    let mut ordered = samples.clone();
    ordered.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    Row {
        slug: workload.slug,
        title: workload.title,
        iters,
        best_ms: ordered.first().copied().unwrap_or(0.0),
        median_ms: ordered.get(REPEATS / 2).copied().unwrap_or(0.0),
        samples_ms: samples,
        runs,
        checksum: bench.checksum(),
        cmds: None,
        nodes_leaked: 0,
    }
    // `bench` drops here, before the next workload builds its graph, so workloads never
    // overlap in the heap.
}

/// One line per workload, `|`-separated: the test parses this instead of hunting for
/// per-workload element ids, so adding a workload does not touch the test.
///
/// Eight fields, or eleven when the workload was command-counted. The optional tail stays the
/// tail, so the sample list is at a fixed index either way. All-or-nothing per suite, never per
/// row, so a parser can decide on the field count once.
pub fn report_text(rows: &[Row]) -> String {
    let mut out = String::new();
    for row in rows {
        let samples = row
            .samples_ms
            .iter()
            .map(|sample| format!("{sample:.4}"))
            .collect::<Vec<_>>()
            .join(";");

        out.push_str(&format!(
            "{}|{}|{:.3}|{:.3}|{:.4}|{}|{}|{samples}",
            row.slug,
            row.iters,
            row.best_ms,
            row.median_ms,
            row.per_op_us(),
            row.runs,
            row.checksum,
        ));
        if let Some(cmds) = &row.cmds {
            out.push_str(&format!(
                "|{}|{}|{}",
                cmds.total,
                row.nodes_leaked,
                cmds.breakdown()
            ));
        }
        out.push('\n');
    }
    out
}
