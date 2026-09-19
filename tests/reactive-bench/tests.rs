//! Runs the reactive-graph benchmark in a real browser, prints the results, and compares them
//! against an earlier run.
//!
//! Requires a WebDriver on localhost:9515. Run with:
//!
//! ```text
//! task reactive-bench
//! BASELINE=target/bench/reactive/<earlier>.json task reactive-bench-compare
//! ```
//!
//! This is a *reporter*, not a perf gate: it asserts only on things that hold regardless of
//! machine speed (see `assert_row` and the cutoff/fan-out invariants at the bottom). Timings go
//! into the table and the JSON; what is asserted is the graph's semantics.
//!
//! Why the suite exists at all: the graph's recent optimisations were allocation-shaped, and
//! wasm32 uses dlmalloc where native builds use glibc malloc, so a native harness cannot see
//! the difference that matters.

use std::time::Duration;

use fantoccini::{Client, ClientBuilder, Locator};
use fantoccini_tests::{Ctx, TestResult};
use vertigo_bench_report::{
    Artifact, Meta, Metric, Row, Run, Suite, now_unix, parse_samples, scale,
};
use vertigo_cli::{BuildOpts, CommonOpts, ServeOpts, build, serve};

/// Must differ from `basic` (5555): cargo may run the test binaries concurrently.
const PORT: u16 = 5556;
/// Must also differ - `build::run` starts by wiping its dest dir, so sharing `./build` would
/// let one test delete the other's artifacts.
const DEST_DIR: &str = "./build-reactive-bench";
const PACKAGE: &str = "vertigo-test-reactive-bench";
const RUN_TIMEOUT: Duration = Duration::from_secs(300);

/// Batches per workload, fixed in `vertigo-bench-support`'s runner. Recorded in the harness
/// block so a baseline taken with a different number says so rather than being subtracted.
const REPEATS: usize = 3;

const SUITE: Suite = Suite {
    kind: "vertigo-reactive-bench",
    dir: "reactive",
    title: "reactive graph, WASM in a browser",
    metrics: &[
        Metric {
            name: "batch_ms",
            unit: "ms",
            floor: 1.0,
        },
        Metric {
            name: "per_op_us",
            unit: "us",
            floor: 0.005,
        },
    ],
    headline: "batch_ms",
    counters: &["iters", "runs", "checksum"],
    // Three batches on a browser main thread, with GC and the compositor in the way. Wider than
    // the SSR suite's band for the same reason the hydration suite's is.
    noise_band_pct: 5.0,
    instability_ratio: 1.5,
    instability_floor: 2.0,
    baseline_env: "VERTIGO_REACTIVE_BENCH_BASELINE",
    label_env: "VERTIGO_REACTIVE_BENCH_LABEL",
};

/// Every workload expected in the report. Catches one being dropped from the table silently.
const EXPECTED_SLUGS: &[&str] = &[
    "list-edit",
    "wide-aggregate",
    "deep-chain",
    "cutoff-fanout",
    "full-fanout",
    "build-teardown",
    "clock-roundtrip",
];

const FANOUT: u64 = 10_000;

#[derive(Debug)]
struct Reported {
    slug: String,
    iters: u64,
    best_ms: f64,
    per_op_us: f64,
    runs: u64,
    checksum: u64,
    samples_ms: Vec<f64>,
}

#[tokio::test]
#[ignore]
async fn reactive_bench() -> TestResult {
    // Go to project root
    let _ = std::env::set_current_dir("..");

    println!("Building {PACKAGE}");

    let opts = BuildOpts {
        common: CommonOpts {
            dest_dir: DEST_DIR.to_string(),
            log_local_time: None,
        },
        inner: build::BuildOptsInner {
            package_name: Some(PACKAGE.to_string()),
            public_path: None,
            // Both on purpose: the point is to measure the artifact that actually ships.
            wasm_opt: Some(true),
            release_mode: Some(true),
            wasm_run_source_map: false,
            cargo_opts: vec![],
        },
    };

    assert!(build::run(opts).is_ok(), "build failed");

    use tokio::sync::oneshot;
    let (sender, receiver) = oneshot::channel::<i32>();

    println!("Spawning vertigo serve on port {PORT}");

    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        let opts = ServeOpts {
            common: CommonOpts {
                dest_dir: DEST_DIR.to_string(),
                log_local_time: None,
            },
            inner: serve::ServeOptsInner {
                host: "127.0.0.1".into(),
                port: PORT,
                mount_point: "/".to_string(),
                proxy: vec![],
                env: vec![],
                wasm_preload: true,
                disable_hydration: false,
                // Off for the benchmarks: compressing every response measures brotli's
                // throughput as much as vertigo's, and the recorded baselines in
                // `target/bench/` were taken without it.
                disable_compression: true,
                threads: None,
            },
        };

        handle.block_on(async {
            tokio::select! {
                ret = serve::run(opts, None) => {
                    match ret {
                        Ok(()) => 1,
                        Err(err) => {
                            println!("Can't spawn vertigo-cli: {err:?}");
                            1
                        }
                    }
                }
                _ = receiver => { 2 }
            }
        });
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let client = ClientBuilder::native()
        .connect("http://localhost:9515")
        .await
        .ctx("failed to connect to WebDriver - is chromedriver running on :9515?")?;

    // The app reads `?scale=` and multiplies every workload's iteration count by it, which is
    // how a run is made long enough to be readable on a slow machine or short enough to iterate
    // on. Passed through the URL rather than the environment because the app is in the browser.
    let site_url = format!("http://127.0.0.1:{PORT}/?scale={}", scale());
    println!("Opening {site_url}");
    client.goto(&site_url).await.ctx("goto failed")?;

    println!("Waiting for the benchmark to finish (timeout {RUN_TIMEOUT:?})");
    wait_for_done(&client, RUN_TIMEOUT).await;

    let report = text_of(&client, "bench-report")
        .await
        .ctx("#bench-report missing")?;
    let user_agent = text_of(&client, "bench-ua").await.unwrap_or_default();
    let total_ms = text_of(&client, "bench-total-ms").await.unwrap_or_default();

    let rows = parse_report(&report);

    client.close().await.ctx("close failed")?;
    sender.send(1).ok();
    tokio::time::sleep(Duration::from_secs(1)).await;

    // --- the report ---------------------------------------------------------

    let created_at_unix = now_unix();
    let run = build_run(
        &rows,
        user_agent,
        &total_ms,
        Artifact::scan("reactive-bench", DEST_DIR)?,
    );

    run.print();

    let path = run.write_and_compare(created_at_unix)?;

    // --- assertions: only what is machine-independent -----------------------

    for slug in EXPECTED_SLUGS {
        assert!(
            rows.iter().any(|row| row.slug == *slug),
            "workload {slug} missing from the report"
        );
    }

    for row in &rows {
        assert_row(row);
    }

    // The real regression guard: these come out of the graph's semantics, not its speed.
    let cutoff = find_row(&rows, "cutoff-fanout");
    assert_eq!(
        cutoff.runs, 0,
        "a write that leaves the parity unchanged must not recompute any of the {FANOUT} children"
    );

    let full = find_row(&rows, "full-fanout");
    assert_eq!(
        full.runs,
        full.iters * FANOUT,
        "a write that flips the parity must recompute every child exactly once per iteration"
    );

    // Last, and only on the way out: `latest.json` is what the next `-compare` subtracts
    // against by default, so a run that failed the semantic gates above must not become one.
    run.promote_to_latest(&path)?;

    Ok(())
}

fn build_run(
    rows: &[Reported],
    user_agent: String,
    total_ms: &str,
    artifacts: Vec<Artifact>,
) -> Run {
    let mut meta = Meta::collect();
    meta.user_agent = user_agent;
    let label = meta.label_for(&SUITE, meta.label_sha());

    Run::new(&SUITE, meta, label)
        .artifacts(artifacts)
        .header_line(format!("total     : {total_ms} ms for the whole page"))
        .harness("repeats", REPEATS)
        .harness("scale", scale())
        .rows(rows.iter().map(Reported::to_row).collect())
}

impl Reported {
    fn to_row(&self) -> Row {
        let row = Row::new(&self.slug)
            .batch(self.samples_ms.clone(), self.iters)
            .counter("runs", self.runs as i64);

        // Not a measurement: it is the value the workload folded out of its own graph, and it
        // changing means the workload computed something different - which the comparison
        // reports loudly. Recorded only where that statement is true.
        match CLOCK_DERIVED_CHECKSUM.contains(&self.slug.as_str()) {
            true => row,
            false => row.counter("checksum", self.checksum as i64),
        }
    }
}

/// Workloads whose checksum cannot repeat, and so must not be compared.
///
/// Exactly one. `clock-roundtrip` sums `now_ms()`: it exists to price the wasm/JS round trip
/// that the timer itself costs, so its checksum is wall-clock-derived by construction.
/// Comparing it reports `COUNTERS CHANGED - the work itself is different` on every single
/// comparison, which is how a real alarm gets learned as noise and stops being read.
///
/// An exclusion list rather than an inclusion list, so a workload added to `EXPECTED_SLUGS`
/// gets its checksum compared by default and has to opt out deliberately.
const CLOCK_DERIVED_CHECKSUM: &[&str] = &["clock-roundtrip"];

/// `Client::find` does not retry, so poll - and on timeout say what the page was doing, rather
/// than just that an element was missing.
async fn wait_for_done(client: &Client, timeout: Duration) {
    let found = client
        .wait()
        .at_most(timeout)
        .every(Duration::from_millis(250))
        .for_element(Locator::Id("bench-done"))
        .await;

    if let Err(err) = found {
        let status = text_of(client, "bench-status").await;
        let current = text_of(client, "bench-current").await;
        panic!(
            "benchmark did not finish within {timeout:?} \
             (status={status:?}, current workload={current:?}): {err}"
        );
    }
}

async fn text_of(client: &Client, id: &str) -> Option<String> {
    match client.find(Locator::Id(id)).await {
        Ok(element) => element.text().await.ok(),
        // A missing element is an answer; anything else means the session is in trouble and
        // should fail loudly rather than read as "not there".
        Err(err) if err.is_no_such_element() => None,
        Err(err) => panic!("WebDriver failed reading #{id}: {err}"),
    }
}

fn parse_report(report: &str) -> Vec<Reported> {
    report
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split('|').collect();
            assert_eq!(
                fields.len(),
                8,
                "malformed report line {line:?} - expected 8 `|`-separated fields"
            );
            let number = |index: usize| -> f64 {
                fields[index]
                    .parse()
                    .unwrap_or_else(|_| panic!("field {index} of {line:?} is not a number"))
            };
            Reported {
                slug: fields[0].to_string(),
                iters: number(1) as u64,
                best_ms: number(2),
                per_op_us: number(4),
                runs: number(5) as u64,
                checksum: number(6) as u64,
                samples_ms: parse_samples(fields[7], line),
            }
        })
        .collect()
}

fn find_row<'a>(rows: &'a [Reported], slug: &str) -> &'a Reported {
    rows.iter()
        .find(|row| row.slug == slug)
        .unwrap_or_else(|| panic!("no row for {slug}"))
}

fn assert_row(row: &Reported) {
    let Reported {
        slug,
        iters,
        best_ms,
        per_op_us,
        ..
    } = row;
    assert!(*iters > 0, "{slug}: no iterations");
    assert!(
        best_ms.is_finite() && *best_ms > 0.0,
        "{slug}: implausible batch time {best_ms}ms"
    );
    assert!(
        per_op_us.is_finite() && *per_op_us > 0.0,
        "{slug}: implausible per-operation time {per_op_us}us"
    );
}
