//! The graph shapes, transcribed from `crates/vertigo/src/reactive_old/compare.rs` with the
//! same constants so browser and native numbers describe the same graphs.
//!
//! One difference is deliberate: `compare.rs` builds on the default graph, these build on an
//! isolated [`Graph`]. The default graph is the one the DOM subscribes to, so writes there
//! would drag vertigo's render machinery into every measurement. The graphs themselves are
//! identical.
//!
//! Every computed is subscribed, which primes it and registers its parent edges. An unprimed
//! computed receives no propagation, and the benchmark would measure nothing.
//!
//! ## Baseline
//!
//! Recorded 2026-08-24 on Linux x86_64 (6.18.12-amd64), Chrome 145.0.7632.109 via
//! ChromeDriver 145, release build with `wasm-opt -Os`. Native column is
//! `task reactive-compare` on the same machine, same day.
//!
//! ```text
//! workload           per op (wasm)   per op (native)   wasm/native
//! list-edit               35.9 us          ~40 us          0.90
//! wide-aggregate           7.6 us           ~9 us          0.84
//! deep-chain              41.9 us          ~32 us          1.31
//! full-fanout            3270 us         ~5900 us          0.55
//! cutoff-fanout           0.154 us              -             -
//! build-teardown         1252 us               -             -
//! clock-roundtrip           4.1 us              -             -
//! ```
//!
//! The headline: **no dlmalloc penalty is visible**. Three of the four comparable shapes
//! run faster in the browser than natively, and only the deep chain is meaningfully slower.
//! The allocation-shaped optimisations that motivated this benchmark hold up on the target
//! that ships.
//!
//! Two numbers have no native counterpart worth quoting. `cutoff-fanout` is ~0.15us per
//! operation; the native harness times a single write with `Instant`, which at that scale
//! measures its own overhead - averaging over 800k operations here is the trustworthy
//! figure. `build-teardown` has no native equivalent at all.
//!
//! `clock-roundtrip` prices the measurement itself: ~4us per wasm->JS round trip, two per
//! batch, against batches of 100ms+. That is the justification for timing batches rather
//! than individual operations.
//!
//! Iteration counts are tuned so every batch lands in 100-300ms on that machine. Re-tune
//! them if a batch drifts far outside that band; `?scale=` shortens a run without changing
//! the per-operation figures.

use std::{cell::Cell, rc::Rc};

use vertigo::{Computed, DropResource, Value, reactive::Graph};

use vertigo_bench_support::{Bench, Workload, now_ms};

const ITEMS: usize = 500;
const CHAIN: usize = 200;
const FANOUT: usize = 10_000;

pub const WORKLOADS: &[Workload] = &[
    Workload {
        slug: "list-edit",
        title: "500-item list, edit one quantity",
        iters: 4_000,
        make: make_list_edit,
    },
    Workload {
        slug: "wide-aggregate",
        title: "aggregate over 500 values, write one",
        iters: 20_000,
        make: make_wide_aggregate,
    },
    Workload {
        slug: "deep-chain",
        title: "chain of 200 computeds, write the source",
        iters: 3_000,
        make: make_deep_chain,
    },
    Workload {
        slug: "cutoff-fanout",
        title: "10k fan-out, write absorbed by the cutoff",
        iters: 800_000,
        make: make_cutoff_fanout,
    },
    Workload {
        slug: "full-fanout",
        title: "10k fan-out, every child recomputes",
        iters: 50,
        make: make_full_fanout,
    },
    Workload {
        slug: "build-teardown",
        title: "build, prime and drop the 500-item list graph",
        iters: 120,
        make: make_build_teardown,
    },
    Workload {
        // Left short on purpose: this prices the timer, it is not a graph measurement.
        slug: "clock-roundtrip",
        title: "diagnostic: one wasm->JS round trip",
        iters: 2_000,
        make: make_clock_roundtrip,
    },
];

/// Bumped by every compute closure, and reported into the DOM. This is what makes the work
/// observable: neither LLVM nor `wasm-opt -Os` can drop a computation whose side effect
/// reaches a rendered element.
type Runs = Rc<Cell<u64>>;

fn counter() -> Runs {
    Rc::new(Cell::new(0))
}

fn bump(runs: &Runs) {
    runs.set(runs.get() + 1);
}

// -- 1. list widget ----------------------------------------------------------

struct ListGraph {
    graph: Graph,
    quantities: Vec<Value<u64>>,
    footer: Computed<String>,
    runs: Runs,
    _subs: Vec<DropResource>,
}

/// The `list-edit` scenario from `compare.rs`: per-item line total and row, a money total
/// over all items, and a selection branch the quantity edit never touches.
fn build_list(runs: &Runs) -> ListGraph {
    let graph = Graph::new();

    let labels: Vec<Value<String>> = (0..ITEMS)
        .map(|i| graph.value(format!("Item {i}")))
        .collect();
    let prices: Vec<Value<u64>> = (0..ITEMS).map(|i| graph.value(100 + i as u64)).collect();
    let quantities: Vec<Value<u64>> = (0..ITEMS).map(|_| graph.value(1)).collect();
    let selected: Vec<Value<bool>> = (0..ITEMS).map(|i| graph.value(i == 0)).collect();

    let mut subs = Vec::new();
    let mut line_totals = Vec::new();

    for index in 0..ITEMS {
        let line_total = graph.computed({
            let price = prices[index].clone();
            let qty = quantities[index].clone();
            let runs = runs.clone();
            move |ctx| {
                bump(&runs);
                price.get(ctx) * qty.get(ctx)
            }
        });

        let row = graph.computed({
            let label = labels[index].clone();
            let qty = quantities[index].clone();
            let selected = selected[index].clone();
            let line_total = line_total.clone();
            let runs = runs.clone();
            move |ctx| {
                bump(&runs);
                let mark = if selected.get(ctx) { "[x]" } else { "[ ]" };
                format!(
                    "{mark} {} x{} = {}",
                    label.get(ctx),
                    qty.get(ctx),
                    line_total.get(ctx)
                )
            }
        });

        subs.push(row.subscribe(|_| {}));
        line_totals.push(line_total);
    }

    let total = graph.computed({
        let runs = runs.clone();
        move |ctx| {
            bump(&runs);
            line_totals.iter().map(|line| line.get(ctx)).sum::<u64>()
        }
    });

    let footer = graph.computed({
        let total = total.clone();
        let runs = runs.clone();
        move |ctx| {
            bump(&runs);
            format!("Total: {}", total.get(ctx))
        }
    });
    subs.push(footer.clone().subscribe(|_| {}));

    let selected_count = graph.computed({
        let runs = runs.clone();
        move |ctx| {
            bump(&runs);
            selected.iter().filter(|flag| flag.get(ctx)).count()
        }
    });

    let any_selected = graph.computed({
        let runs = runs.clone();
        move |ctx| {
            bump(&runs);
            selected_count.get(ctx) > 0
        }
    });

    let toolbar = graph.computed({
        let runs = runs.clone();
        move |ctx| {
            bump(&runs);
            if any_selected.get(ctx) {
                "Delete (enabled)"
            } else {
                "Delete (disabled)"
            }
        }
    });
    subs.push(toolbar.subscribe(|_| {}));

    ListGraph {
        graph,
        quantities,
        footer,
        runs: runs.clone(),
        _subs: subs,
    }
}

struct ListEdit {
    list: ListGraph,
    step: Cell<u32>,
}

fn make_list_edit() -> Box<dyn Bench> {
    Box::new(ListEdit {
        list: build_list(&counter()),
        step: Cell::new(0),
    })
}

impl Bench for ListEdit {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = self.step.get();
            self.step.set(step.wrapping_add(1));
            let index = step as usize % self.list.quantities.len();
            self.list.quantities[index].set(u64::from(step) + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.list.runs.replace(0)
    }

    fn checksum(&self) -> u64 {
        let footer = self.list.graph.transaction(|ctx| self.list.footer.get(ctx));
        footer.len() as u64
    }
}

// -- 2. wide aggregate -------------------------------------------------------

struct WideAggregate {
    graph: Graph,
    leaves: Vec<Value<u64>>,
    total: Computed<u64>,
    runs: Runs,
    step: Cell<u32>,
    _subs: Vec<DropResource>,
}

fn make_wide_aggregate() -> Box<dyn Bench> {
    let graph = Graph::new();
    let runs = counter();
    let leaves: Vec<Value<u64>> = (0..ITEMS).map(|i| graph.value(i as u64)).collect();

    let total = graph.computed({
        let leaves = leaves.clone();
        let runs = runs.clone();
        move |ctx| {
            bump(&runs);
            leaves.iter().map(|leaf| leaf.get(ctx)).sum::<u64>()
        }
    });

    let subs = vec![total.clone().subscribe(|_| {})];

    Box::new(WideAggregate {
        graph,
        leaves,
        total,
        runs,
        step: Cell::new(0),
        _subs: subs,
    })
}

impl Bench for WideAggregate {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = self.step.get();
            self.step.set(step.wrapping_add(1));
            let index = step as usize % self.leaves.len();
            self.leaves[index].set(u64::from(step) + 1_000);
        }
    }

    fn take_runs(&self) -> u64 {
        self.runs.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.graph.transaction(|ctx| self.total.get(ctx))
    }
}

// -- 3. deep chain -----------------------------------------------------------

struct DeepChain {
    graph: Graph,
    source: Value<u64>,
    tip: Computed<u64>,
    runs: Runs,
    step: Cell<u32>,
    _subs: Vec<DropResource>,
}

fn make_deep_chain() -> Box<dyn Bench> {
    let graph = Graph::new();
    let runs = counter();
    let source = graph.value(0u64);

    let mut prev: Computed<u64> = source.to_computed();
    let mut subs = Vec::new();
    for _ in 0..CHAIN {
        let next = graph.computed({
            let prev = prev.clone();
            let runs = runs.clone();
            move |ctx| {
                bump(&runs);
                prev.get(ctx) + 1
            }
        });
        subs.push(next.clone().subscribe(|_| {}));
        prev = next;
    }

    Box::new(DeepChain {
        graph,
        source,
        tip: prev,
        runs,
        step: Cell::new(0),
        _subs: subs,
    })
}

impl Bench for DeepChain {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = self.step.get();
            self.step.set(step.wrapping_add(1));
            self.source.set(u64::from(step) + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.runs.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.graph.transaction(|ctx| self.tip.get(ctx))
    }
}

// -- 4/5. fan-out, cut off and not -------------------------------------------

struct Fanout {
    graph: Graph,
    source: Value<u64>,
    parity: Computed<bool>,
    /// Counts only the 10k children, so `cutoff` can assert an exact zero without the
    /// parity node's own recompute showing up.
    child_runs: Runs,
    step: Cell<u32>,
    /// `false` keeps the parity constant, so the write is absorbed by the equality cutoff;
    /// `true` flips it on every write, so all 10k children recompute.
    flip_parity: bool,
    _subs: Vec<DropResource>,
}

fn make_fanout(flip_parity: bool) -> Box<dyn Bench> {
    let graph = Graph::new();
    let child_runs = counter();
    let source = graph.value(1u64);

    let parity = graph.computed({
        let source = source.clone();
        move |ctx| source.get(ctx).is_multiple_of(2)
    });

    let subs = (0..FANOUT)
        .map(|i| {
            let parity = parity.clone();
            let runs = child_runs.clone();
            graph
                .computed(move |ctx| {
                    // One `Cell` bump per child. For `full-fanout` that is 10k bumps against
                    // a multi-millisecond iteration - well under 1%, and constant across
                    // runs, so it does not distort the comparison.
                    bump(&runs);
                    (parity.get(ctx), i)
                })
                .subscribe(|_| {})
        })
        .collect();

    Box::new(Fanout {
        graph,
        source,
        parity,
        child_runs,
        step: Cell::new(0),
        flip_parity,
        _subs: subs,
    })
}

fn make_cutoff_fanout() -> Box<dyn Bench> {
    make_fanout(false)
}

fn make_full_fanout() -> Box<dyn Bench> {
    make_fanout(true)
}

impl Bench for Fanout {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = u64::from(self.step.get());
            self.step.set(self.step.get().wrapping_add(1));
            let next = if self.flip_parity {
                // Alternates odd/even, so `parity` changes and the whole fan-out reruns.
                step
            } else {
                // Always odd: a new value every time, so the write is not swallowed by
                // `Value::set`'s equality check, but `parity` never changes.
                2 * step + 1
            };
            self.source.set(next);
        }
    }

    fn take_runs(&self) -> u64 {
        self.child_runs.replace(0)
    }

    fn checksum(&self) -> u64 {
        u64::from(self.graph.transaction(|ctx| self.parity.get(ctx)))
    }
}

// -- 6. build and tear down --------------------------------------------------

/// Construction is where the graph allocates in bulk - the parent buffers, the node maps,
/// the `Rc` kept per parent edge - so it is the path most likely to diverge between glibc
/// malloc and dlmalloc. It is also a real app path: every route change builds and drops a
/// subtree.
///
/// One iteration builds *and* drops, which keeps the heap in steady state. Retaining the
/// graphs instead would isolate construction, but would grow the heap by tens of megabytes
/// mid-batch and perturb the very allocator behaviour being measured.
struct BuildTeardown {
    runs: Runs,
    sink: Cell<u64>,
}

fn make_build_teardown() -> Box<dyn Bench> {
    Box::new(BuildTeardown {
        runs: counter(),
        sink: Cell::new(0),
    })
}

impl Bench for BuildTeardown {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let list = build_list(&self.runs);
            let footer = list.graph.transaction(|ctx| list.footer.get(ctx));
            self.sink
                .set(self.sink.get().wrapping_add(footer.len() as u64));
            drop(list);
        }
    }

    fn take_runs(&self) -> u64 {
        self.runs.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.sink.get()
    }
}

// -- 7. clock diagnostic -----------------------------------------------------

/// Not a graph measurement: it prices the wasm->JS round trip the timer itself costs, so a
/// reader can tell how much of a short batch is measurement overhead.
struct ClockRoundtrip {
    sink: Cell<u64>,
    calls: Cell<u64>,
}

fn make_clock_roundtrip() -> Box<dyn Bench> {
    Box::new(ClockRoundtrip {
        sink: Cell::new(0),
        calls: Cell::new(0),
    })
}

impl Bench for ClockRoundtrip {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            self.sink.set(self.sink.get().wrapping_add(now_ms() as u64));
            self.calls.set(self.calls.get() + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.calls.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.sink.get()
    }
}
