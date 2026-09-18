//! Measures server-side rendering, per phase, and compares a run against an earlier one.
//!
//! ```text
//! task ssr-bench                                   # run and write target/bench/ssr/<sha>-<ts>.json
//! BASELINE=target/bench/ssr/<earlier>.json task ssr-bench-compare
//! ```
//!
//! Unlike the other three suites in this package there is **no browser and no WebDriver**,
//! and nothing binds a port. SSR is wasmtime plus `vertigo-cli`, both of which run on the
//! host, so this drives `ServerState` in-process and reads the phase breakdown straight out
//! of `request_timed`.
//!
//! ## What is asserted, and what is only printed
//!
//! Timings are **printed, never asserted** - they swing several-fold between machines, and
//! the existing suites already say so. The gates are the things that hold regardless of how
//! fast the machine is:
//!
//! - every expected route present, so one silently dropped from the table fails;
//! - every render of a route byte-identical to every other, which is what proves no guest
//!   state survives a request;
//! - the exact DOM-command count per repeated unit, which is the SSR analogue of
//!   `dom-bench`'s command-count assertions and the real regression guard.
//!
//! The demo half gets the first two but not the third: its markup changes whenever the demo
//! does, and pinning constants to it would produce failures that mean nothing.
//!
//! ## Why the subject app is a dependency rather than a copy
//!
//! `dom-bench` duplicates its scene sizes into its driver and relies on assertions to catch
//! the drift, because a browser suite's app only exists as wasm. Here the driver and the app
//! can share a compilation, so the sizes and the route list are imported and cannot drift.

use std::{collections::BTreeMap, sync::Arc};

use fantoccini_tests::{Ctx, TestResult};
use vertigo_bench_report::{
    Artifact, Meta, Metric, Row, Run, Stats, Suite, body_hash, micros, now_unix,
};
use vertigo_cli::{
    BuildOpts, CommonOpts, build,
    serve::{MountConfigBuilder, ServerState},
};
use vertigo_test_ssr_bench::{route::Route, shapes};

const SUITE: Suite = Suite {
    kind: "vertigo-ssr-bench",
    dir: "ssr",
    title: "SSR render, native host (wasmtime + vertigo-cli), release",
    // The total first because it is the headline, then the phases in the order they happen.
    //
    // Everything after `total` **sums to it**, which is the property that makes the row
    // readable: a phase that grew has to have taken the growth from somewhere, and work that
    // belongs to no phase lands in `unaccounted` rather than nowhere. Keeping that true is why
    // `html_other` and `fetch_wait` have columns of their own - `unaccounted` is measured as
    // the total minus `build_response`, so the slack inside `build_response` that the three
    // `html_*` marks do not cover would otherwise cancel out of the arithmetic and a new
    // untimed step there would move no column at all.
    //
    // `unaccounted` large and positive therefore means a phase is missing a timer;
    // zero-and-suspicious means something is being counted twice.
    metrics: &[
        US("total"),
        US("instantiate"),
        US("wasm_exec"),
        US("host_wire"),
        US("decode"),
        US("feed"),
        US("html_tree"),
        US("html_inject"),
        US("html_string"),
        US("html_other"),
        US("fetch_wait"),
        US("unaccounted"),
    ],
    headline: "total",
    counters: &[
        "status",
        "commands",
        "batches",
        "html_bytes",
        "blob_bytes",
        "host_calls",
        "wasm_calls",
    ],
    // `dom-bench`'s notes already record several percent of run-to-run spread on an idle
    // machine, and running this suite twice on the same commit puts almost every total inside
    // 3%. Deliberately conservative: a benchmark that cries wolf gets ignored.
    noise_band_pct: 3.0,
    instability_ratio: 1.25,
    // In microseconds. An in-process render is orders of magnitude shorter than a page load,
    // so this is much tighter than the browser suites' floor.
    instability_floor: 50.0,
    baseline_env: "VERTIGO_SSR_BENCH_BASELINE",
    label_env: "VERTIGO_SSR_BENCH_LABEL",
};

/// A microsecond phase with the suite's shared absolute noise floor.
///
/// The percentage bar alone is not enough here: `html_inject` is two or three microseconds on
/// most routes, so a scheduler hiccup of one microsecond is a 40% "regression", and two runs of
/// the same commit produced exactly that.
#[allow(non_snake_case)]
const fn US(name: &'static str) -> Metric {
    Metric {
        name,
        unit: "us",
        floor: 10.0,
    }
}

/// The deterministic half of a render. A change here is a change in what was rendered, which is
/// a different thing from a change in how fast it rendered - and has to be read first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counters {
    batches: u32,
    commands: u32,
    blob_bytes: u64,
    html_bytes: u64,
    host_calls: u32,
    wasm_calls: u32,
}

/// Distinct from `./build`, `./build-reactive-bench`, `./build-dom-bench` and
/// `./build-demo`, and from each other: `build::run` wipes its dest dir on entry, so a
/// shared one would let two suites delete each other's artifacts.
const SSR_DEST_DIR: &str = "./build-ssr-bench";
const DEMO_DEST_DIR: &str = "./build-ssr-demo";

const SSR_PACKAGE: &str = "vertigo-test-ssr-bench";
const DEMO_PACKAGE: &str = "vertigo-demo";

/// Both non-root, and different from each other.
///
/// Different because `ServerState`'s global registry is keyed by mount point, which is what
/// lets two apps live in one process. Both non-root so both take the same branch of the
/// mount-point substitution in `build_response` - mounting one at `/` would make the two
/// subjects incomparable for no reason at all.
const SSR_MOUNT: &str = "/ssr-bench";
const DEMO_MOUNT: &str = "/demo";

/// Discarded. Absorbs wasmtime's first instantiation of a module, first-touch of the host
/// allocator, and the page cache for the wasm file.
const WARMUP: usize = 3;
/// Timed renders per route. `VERTIGO_SSR_BENCH_SAMPLES` overrides it - the host-side
/// equivalent of `?scale=` in the browser suites.
const SAMPLES: usize = 15;

/// Demo routes that are safe to render server-side.
///
/// Excluded, and why: `/fetch` and `/lazy-list` read a `LazyCache` during render, which
/// issues an SSR fetch through `actix_web::rt::spawn` - that is `spawn_local` and panics
/// outside a `LocalSet`. `/chat` and `/ws-collection` are websocket tabs that render only
/// their "turned off" message without the env vars, so there would be nothing to measure.
/// `/github_explorer` is safe despite the name: it fetches only once a repository has been
/// picked, and nothing has been.
const DEMO_ROUTES: &[&str] = &[
    "/",
    "/counters",
    "/styling",
    "/sudoku",
    "/input",
    "/github_explorer",
    "/game_of_life",
    "/driver",
    "/js-api-access",
    "/list",
    "/svg",
];

/// One render's numbers.
struct Sample {
    total: f64,
    phases: BTreeMap<String, f64>,
    counters: Counters,
    status: u16,
    hash: u64,
}

#[tokio::test]
#[ignore]
async fn ssr_bench() -> TestResult {
    // A debug host measures Cranelift and an unoptimised wasmtime, not vertigo. Refused
    // rather than warned about, so a debug run can never be written to a file and later
    // compared against a release one.
    //
    // A runtime check rather than an `assert!`, which clippy rightly calls a constant
    // assertion - and `const { assert!(..) }`, the suggested fix, would fail the debug build
    // of the whole package rather than this one test.
    if cfg!(debug_assertions) {
        return Err(
            "build this suite with --release, or use `task ssr-bench` - a debug host \
                    measures wasmtime rather than vertigo"
                .into(),
        );
    }

    // Process-global, which is why this binary holds exactly one test: a second would run
    // on another thread in the same process and find the working directory already moved.
    let _ = std::env::set_current_dir("..");

    let samples = std::env::var("VERTIGO_SSR_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(SAMPLES);

    let skip_build = std::env::var("VERTIGO_SSR_BENCH_SKIP_BUILD").is_ok();

    let ssr = subject(
        "ssr-bench",
        SSR_PACKAGE,
        SSR_DEST_DIR,
        SSR_MOUNT,
        skip_build,
    )?;
    let demo = subject("demo", DEMO_PACKAGE, DEMO_DEST_DIR, DEMO_MOUNT, skip_build)?;

    let ssr_state = ServerState::global(SSR_MOUNT);
    let demo_state = ServerState::global(DEMO_MOUNT);

    // (subject, route, state). Built once and walked in order, repeatedly.
    let mut targets: Vec<(&str, String, Arc<ServerState>)> = Vec::new();
    for route in Route::ALL {
        targets.push(("ssr-bench", route.path().to_string(), ssr_state.clone()));
    }
    targets.push(("ssr-bench", "/plain.txt".to_string(), ssr_state.clone()));
    for route in DEMO_ROUTES {
        targets.push(("demo", (*route).to_string(), demo_state.clone()));
    }

    println!(
        "Rendering {} routes, {WARMUP} warmup + {samples} timed each",
        targets.len()
    );

    // Round-robin rather than route-by-route. Every SSR render is independent - unlike the
    // browser suites, where a workload owns a mounted scene - so interleaving spreads
    // thermal drift and background load evenly instead of concentrating it on whichever
    // route happened to run while something else was busy.
    let mut collected: BTreeMap<String, Vec<Sample>> = BTreeMap::new();

    for pass in 0..(WARMUP + samples) {
        for (subject_name, route, state) in &targets {
            let sample = render(state, route).await;

            if pass >= WARMUP {
                collected
                    .entry(format!("{subject_name}\u{1}{route}"))
                    .or_default()
                    .push(sample);
            }
        }
    }

    let rows = summarise(&targets, &collected)?;

    let created_at_unix = now_unix();

    let meta = Meta::collect();
    let label = meta.label_for(&SUITE, meta.label_sha());
    let run = Run::new(&SUITE, meta, label)
        .harness("warmup", WARMUP)
        .harness("samples", samples)
        .harness("order", "round-robin")
        .harness("skipped_build", skip_build)
        .note("subjects", serde_json::json!([ssr.note(), demo.note()]))
        .header_line(ssr.header_line())
        .header_line(demo.header_line())
        .artifacts(ssr.artifacts.clone())
        .artifacts(demo.artifacts.clone())
        .rows(rows);

    run.print();

    let path = run.write_and_compare(created_at_unix)?;

    assert_rows(&run)?;

    // Last, and only on the way out: `latest.json` is what the next `-compare` subtracts
    // against by default, and a run that failed the gates above measured a command stream this
    // suite says is wrong. Promoting it would hide the regression in the baseline.
    run.promote_to_latest(&path)?;

    Ok(())
}

// --------------------------------------------------------------------------------------
// setup
// --------------------------------------------------------------------------------------

/// One built guest, and what it cost to get it ready.
///
/// Reported rather than guarded: the size of the wasm and how long it took to compile are
/// exactly the things a change is expected to move, so warning when they differ between two
/// runs would fire on every comparison worth making.
struct Subject {
    name: String,
    package: String,
    dest_dir: String,
    mount_point: String,
    compile_us: f64,
    /// What this subject shipped. Reported by the shared artifact block, which also carries the
    /// section breakdown and the gzipped size - so the size is not repeated here.
    artifacts: Vec<Artifact>,
}

impl Subject {
    fn header_line(&self) -> String {
        format!(
            "guest     : {:<10} compiled by wasmtime in {:.0} ms",
            self.name,
            self.compile_us / 1000.0
        )
    }

    fn note(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "package": self.package,
            "dest_dir": self.dest_dir,
            "mount_point": self.mount_point,
            "module_compile_us": self.compile_us,
        })
    }
}

fn subject(
    name: &str,
    package: &str,
    dest_dir: &str,
    mount_point: &str,
    skip_build: bool,
) -> TestResult<Subject> {
    if skip_build {
        println!("Skipping the build of {package} (VERTIGO_SSR_BENCH_SKIP_BUILD)");
    } else {
        println!("Building {package}");

        let opts = BuildOpts {
            common: CommonOpts {
                dest_dir: dest_dir.to_string(),
                log_local_time: None,
            },
            inner: build::BuildOptsInner {
                package_name: Some(package.to_string()),
                public_path: None,
                // Both on purpose: the point is to measure the artifact that actually
                // ships. Release also matters for the command counts - a debug build emits
                // extra `v-component` and `v-css` attribute commands per instance.
                wasm_opt: Some(true),
                release_mode: Some(true),
                wasm_run_source_map: false,
                cargo_opts: vec![],
            },
        };

        build::run(opts).ctx(format!("building {package}"))?;
    }

    let mount_config = MountConfigBuilder::new(mount_point, dest_dir)
        .wasm_preload(true)
        .build()
        .ctx(format!("reading {dest_dir}/index.json"))?;

    // Compiles the module once. Everything after this only instantiates it.
    ServerState::init(&mount_config).ctx(format!("initialising {mount_point}"))?;

    let compile_us = micros(ServerState::global(mount_point).module_compile_time());

    Ok(Subject {
        name: name.to_string(),
        package: package.to_string(),
        dest_dir: dest_dir.to_string(),
        mount_point: mount_point.to_string(),
        compile_us,
        // Read even when the build was skipped: these describe what was actually served, and
        // the `skipped_build` harness flag already warns that it may be stale.
        artifacts: Artifact::scan(name, dest_dir)?,
    })
}

// --------------------------------------------------------------------------------------
// one render
// --------------------------------------------------------------------------------------

async fn render(state: &Arc<ServerState>, route: &str) -> Sample {
    let (response, timings) = state.request_timed(route).await;

    let mut phases = BTreeMap::new();
    let mut set = |name: &str, value| {
        phases.insert(name.to_string(), micros(value));
    };

    set("instantiate", timings.instantiate);
    set("wasm_exec", timings.wasm_self());
    // Host work inside the wasm call that is *not* the command decode: copying the argument
    // out of linear memory, decoding the envelope, encoding the reply back in. Subtracted
    // out of `wasm_exec` along with `decode`, so without a column of its own it would
    // belong to no phase and quietly inflate `unaccounted`.
    set(
        "host_wire",
        timings.host_in_wasm.saturating_sub(timings.decode_dom),
    );
    set("decode", timings.decode_dom);
    set("feed", timings.dom_apply);
    set("html_tree", timings.html_tree);
    set("html_inject", timings.html_inject);
    set("html_string", timings.html_string);
    // The slack inside `build_response` the three marks above do not cover: the body's
    // `String::into_bytes`, the `<html>`/`<body>` validation, the error-path branches.
    // `unaccounted` subtracts the whole of `build_response`, so without this column a new
    // untimed step in there would raise `total` and `build_response` by the same amount and
    // move nothing a reader can see.
    set(
        "html_other",
        timings.build_response.saturating_sub(timings.html_total()),
    );
    // Waiting for an SSR fetch, not working. Also subtracted by `unaccounted`, so it needs a
    // column for the same reason - and it must not be read as time spent rendering.
    set("fetch_wait", timings.fetch_wait);
    set("unaccounted", timings.unaccounted());

    Sample {
        total: micros(timings.total),
        phases,
        counters: Counters {
            batches: timings.dom_batches,
            commands: timings.dom_commands,
            blob_bytes: timings.dom_blob_bytes,
            html_bytes: timings.html_bytes,
            host_calls: timings.host_calls,
            wasm_calls: timings.wasm_calls,
        },
        status: response.status,
        hash: body_hash(&response.body),
    }
}

// --------------------------------------------------------------------------------------
// aggregation
// --------------------------------------------------------------------------------------

fn summarise(
    targets: &[(&str, String, Arc<ServerState>)],
    collected: &BTreeMap<String, Vec<Sample>>,
) -> TestResult<Vec<Row>> {
    let mut rows = Vec::new();

    for (subject, route, _) in targets {
        let key = format!("{subject}\u{1}{route}");
        let samples = collected
            .get(&key)
            .ctx(format!("no samples were collected for {subject} {route}"))?;

        let first = samples
            .first()
            .ctx(format!("no samples were collected for {subject} {route}"))?;

        // Determinism, checked here rather than in `assert_rows` because this is where the
        // individual bodies are still in hand. A route whose renders differ is a route
        // whose timings mean nothing, so say which one and stop.
        for (index, sample) in samples.iter().enumerate() {
            assert_eq!(
                sample.hash, first.hash,
                "{subject} {route}: render {index} produced a different body from render 0. \
                 SSR is supposed to be a pure function of the URL, so this is either guest \
                 state surviving a request or something in the page reading the clock"
            );
            assert_eq!(
                sample.counters, first.counters,
                "{subject} {route}: render {index} emitted different counters from render 0"
            );
        }

        let mut row = Row::new(format!("{subject} {route}"))
            .samples(samples.len())
            .metric_stats(
                "total",
                Stats::of(samples.iter().map(|sample| sample.total).collect()),
            )
            .counter("status", i64::from(first.status))
            .counter("batches", i64::from(first.counters.batches))
            .counter("commands", i64::from(first.counters.commands))
            .counter("blob_bytes", first.counters.blob_bytes as i64)
            .counter("html_bytes", first.counters.html_bytes as i64)
            .counter("host_calls", i64::from(first.counters.host_calls))
            .counter("wasm_calls", i64::from(first.counters.wasm_calls));

        for metric in SUITE.metrics {
            if metric.name == "total" {
                continue;
            }

            row = row.metric(
                metric.name,
                samples
                    .iter()
                    .map(|sample| sample.phases.get(metric.name).copied().unwrap_or(0.0))
                    .collect(),
            );
        }

        rows.push(row);
    }

    Ok(rows)
}

// --------------------------------------------------------------------------------------
// the gates
// --------------------------------------------------------------------------------------

fn assert_rows(run: &Run) -> TestResult {
    let find = |subject: &str, route: &str| -> TestResult<&Row> {
        let key = format!("{subject} {route}");
        run.rows
            .iter()
            .find(|row| row.key == key)
            .ctx(format!("{key} is missing from the report"))
    };

    // Every route the app knows about, so one dropped from the table fails rather than
    // quietly narrowing what the suite covers.
    for route in Route::ALL {
        let row = find("ssr-bench", route.path())?;
        assert_eq!(row.get_counter("status"), 200, "ssr-bench {}", route.path());
        assert!(
            row.get_counter("html_bytes") > 0,
            "ssr-bench {}: empty body",
            route.path()
        );
        assert_eq!(
            row.get_counter("batches"),
            1,
            "ssr-bench {}: the mount must reach the host as a single DOM batch, not {}",
            route.path(),
            row.get_counter("batches")
        );
    }
    for route in DEMO_ROUTES {
        find("demo", route)?;
    }

    let commands = |route: &str| -> TestResult<u32> {
        Ok(find("ssr-bench", route)?.get_counter("commands") as u32)
    };
    let html_bytes = |route: &str| -> TestResult<u64> {
        Ok(find("ssr-bench", route)?.get_counter("html_bytes") as u64)
    };

    // The shell every page renders. Everything below is a difference from it, so no page's
    // expectation has to know what the shell costs, and a change to the shell moves one
    // number rather than five.
    let shell = commands("/")?;

    // Exact, deterministic, machine-independent: the real regression guard, and the SSR
    // analogue of `dom-bench`'s command-count assertions. The per-unit figures were read
    // off a run rather than derived - a change here means the renderer emits a different
    // command stream than it used to, which is worth knowing whatever it did to the
    // timings.
    //
    // Signed, although a count cannot be negative: this suite only ever runs `--release`,
    // where overflow checks are off, so an unsigned `total - shell` on a page that emitted
    // *fewer* commands than the shell wraps to about four billion - and the failure message
    // then points at the per-unit cost rather than at the fact that the page shrank.
    let unit = |route: &str, units: u32, expected: u32| -> TestResult {
        let over = i64::from(commands(route)?) - i64::from(shell);
        assert_eq!(
            over,
            i64::from(units) * i64::from(expected),
            "{route}: {units} units emitted {over} commands over the shell, i.e. {} each rather \
             than {expected}",
            over as f64 / f64::from(units),
        );
        Ok(())
    };

    unit("/wide", shapes::WIDE_N, WIDE_PER_UNIT)?;
    unit("/deep", shapes::DEEP_D, DEEP_PER_UNIT)?;
    unit("/text", shapes::TEXT_N, TEXT_PER_UNIT)?;
    unit("/attrs", shapes::ATTR_N, ATTRS_PER_UNIT)?;
    unit("/css", shapes::CSS_N, CSS_PER_UNIT)?;
    unit("/table", shapes::TABLE_ROWS, TABLE_PER_ROW)?;

    // `/deep` and `/deep-indent` are the same tree; only the pretty-printer differs.
    assert_eq!(
        commands("/deep")?,
        commands("/deep-indent")?,
        "indentation is a serialisation concern and must not change the command stream"
    );
    assert!(
        html_bytes("/deep-indent")? > html_bytes("/deep")?,
        "...but it must change the output, or `<pre>` is not suppressing anything and the \
         pair prices nothing"
    );

    // `/text` and `/text-plain` likewise: same nodes, same text lengths, different bytes.
    assert_eq!(
        commands("/text")?,
        commands("/text-plain")?,
        "escaping is a serialisation concern and must not change the command stream"
    );
    assert!(
        html_bytes("/text")? > html_bytes("/text-plain")?,
        "...but it must change the output, or the escapable characters never arrived"
    );

    // A driver round trip is not DOM work. The page costs the shell plus the one text node
    // it folds the crossing count into - a constant, however large `ROUNDTRIP_N` gets.
    assert_eq!(
        i64::from(commands("/roundtrip")?) - i64::from(shell),
        i64::from(ROUNDTRIP_MARKUP),
        "`/roundtrip` should cost a fixed bit of markup and nothing per round trip - a \
         driver round trip emits no DOM commands"
    );
    let roundtrip = find("ssr-bench", "/roundtrip")?;
    assert!(
        roundtrip.get_counter("host_calls") > i64::from(shapes::ROUNDTRIP_N),
        "/roundtrip made {} host calls, fewer than the {} round trips it performs - \
         `is_browser()` has started caching, and this page no longer measures the boundary",
        roundtrip.get_counter("host_calls"),
        shapes::ROUNDTRIP_N
    );

    // The plain-text route answers before the document is serialised - but only after the
    // tree has already been built and shipped, which is what makes it a phase-4 control.
    let plain = find("ssr-bench", "/plain.txt")?;
    assert_eq!(plain.get_counter("status"), 200);
    assert_eq!(
        plain.get_counter("commands"),
        i64::from(shell),
        "/plain.txt should build the same tree as the shell before answering"
    );
    assert_eq!(
        plain.get_counter("html_bytes") as usize,
        shapes::PLAIN_BODY.len(),
        "/plain.txt should answer with the plain body, not a document"
    );

    Ok(())
}

/// DOM commands one repeated unit of each page costs, over and above the shell.
///
/// Read off a run rather than derived: the exact command stream for a given bit of markup
/// is the renderer's business, and predicting it here would only encode a guess. Expressed
/// per unit so changing a size in `shapes.rs` does not touch these.
const WIDE_PER_UNIT: u32 = 5;
const DEEP_PER_UNIT: u32 = 2;
const TEXT_PER_UNIT: u32 = 4;
const ATTRS_PER_UNIT: u32 = 17;
const CSS_PER_UNIT: u32 = 6;
const TABLE_PER_ROW: u32 = 43;
/// The one text node `/roundtrip` renders, over the shell. Independent of `ROUNDTRIP_N`.
const ROUNDTRIP_MARKUP: u32 = 2;
