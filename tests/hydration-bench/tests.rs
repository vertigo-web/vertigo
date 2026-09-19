//! Measures hydration in a real browser, and compares one implementation against another.
//!
//! Requires a WebDriver on localhost:9515. Run with:
//!
//! ```text
//! task hydration-bench
//! BASELINE=target/bench/hydration/<earlier>.json task hydration-bench-compare
//! ```
//!
//! ## What this is for
//!
//! Hydration has been rewritten from JavaScript to Rust on a separate branch. This suite
//! runs against either, unchanged, so the two can be put side by side.
//!
//! It measures only through things that predate both implementations and are identical on
//! both, so that **nothing in `crates/vertigo` has to be modified on either branch**:
//!
//! - an inline `<script>` this suite's own app server-renders into `<head>`, which installs
//!   a `MutationObserver` (see `src/probe.rs`);
//! - `performance.now()`, read by the app at the first and last line of its render and
//!   carried out on the sentinel element's attributes;
//! - `window.__vertigo_hydration`, whose five field names are an older contract than either
//!   implementation.
//!
//! Instrumentation added separately to two implementations is a thing the comparison could
//! be an artifact of. This way it cannot be.
//!
//! ## What is asserted, and what is only printed
//!
//! Timings are printed, never asserted - the existing suites already say why. The gates are
//! the deterministic facts:
//!
//! - every expected row present;
//! - `matched == hydratable` on every clean-match page, i.e. the server's markup was adopted
//!   rather than rebuilt;
//! - at least two attribute mutations per hydrated page, which is the floor the sentinel
//!   imposes on any implementation and the check that the observer is alive.
//!
//! ## Reading the `[off]` rows, and what `mutations` is
//!
//! `mutations` counts **operations on nodes already in the document**, not nodes touched. A
//! `MutationObserver` reports one `childList` record with one added node when a subtree is
//! attached, however large that subtree is - so work done on detached nodes is nearly free to
//! this metric.
//!
//! That makes the `[off]` rows look astonishingly cheap: rebuilding `/wide` from scratch
//! shows 27 mutations, because vertigo builds the tree detached and attaches it once.
//! Hydrating the same page shows thousands, because every operation lands on a server-built
//! node that is already attached. **The two are therefore not comparable to each other**, and
//! no assertion here compares them. The `[off]` rows are a floor for the timings, not a
//! mutation baseline.
//!
//! Between the two *hydration implementations* the metric is exactly fair: both operate on
//! the same attached server nodes, and the count is the number of times the browser was asked
//! to do something to the document it had already built.
//!
//! ## The demo rows carry coverage only
//!
//! The probe is rendered by this suite's app. `vertigo-demo` cannot render it, and a page's
//! own markup is the only way to get a script in front of wasm across a real navigation - so
//! demo routes have no observer and no marks. They are still worth running: `matched` and
//! `hydratable` come from the framework rather than from the probe, and they answer the
//! question the controlled pages cannot, which is whether a real application hydrates as
//! completely under one implementation as under the other.

use std::{collections::BTreeMap, time::Duration};

use fantoccini::{Client, ClientBuilder};
use fantoccini_tests::{Ctx, TestResult};
use tokio::sync::oneshot;
use vertigo_bench_report::{Artifact, Meta, Metric, Row, Run, Suite, now_unix};
use vertigo_cli::{
    BuildOpts, CommonOpts, ServeOpts, build,
    serve::{self, ServeOptsInner},
};
use vertigo_test_hydration_bench::{SENTINEL_ID, route::Route};

const SUITE: Suite = Suite {
    kind: "vertigo-hydration-bench",
    dir: "hydration",
    title: "hydration, page loads in a real browser",
    // In the order they happen. `render_ms` is the app building its own tree, which is the same
    // work whichever implementation hydrates it - so it is a control: if it moves between two
    // runs, something other than hydration did, and the rest of the table should not be read.
    metrics: &[
        MS("total_ms"),
        MS("boot_ms"),
        MS("render_ms"),
        MS("hydrate_ms"),
    ],
    headline: "total_ms",
    counters: &[
        "matched",
        "hydratable",
        "skipped",
        "total",
        "mutations",
        "added",
        "removed",
        "attrs",
        "chars",
    ],
    // A page load is noisier than an in-process render - browser scheduling, GC and the
    // compositor all land in it - so both bars are wider here than in the SSR suite.
    noise_band_pct: 5.0,
    instability_ratio: 1.5,
    // A row whose total is two milliseconds crosses a 1.5 ratio on a third of a millisecond of
    // scheduler jitter, which is not a busy machine, it is a small number.
    instability_floor: 2.0,
    baseline_env: "VERTIGO_HYDRATION_BENCH_BASELINE",
    label_env: "VERTIGO_HYDRATION_BENCH_LABEL",
};

/// A millisecond phase. Sub-half-millisecond movement in a browser is not a finding.
#[allow(non_snake_case)]
const fn MS(name: &'static str) -> Metric {
    Metric {
        name,
        unit: "ms",
        floor: 0.5,
    }
}

/// What hydration reported about itself, read from `window.__vertigo_hydration`.
///
/// `None` when hydration was switched off: neither implementation publishes a report in that
/// case, which is itself the check that the control really did take the other path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Coverage {
    root_found: bool,
    matched: u64,
    hydratable: u64,
    skipped: u64,
    total: u64,
}

/// DOM mutations observed between arming the observer and the sentinel appearing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Mutations {
    mutations: u64,
    added: u64,
    removed: u64,
    attrs: u64,
    chars: u64,
}

/// Distinct from every other suite's, and from each other: `build::run` wipes its dest dir
/// on entry, so a shared one would let two suites delete each other's artifacts.
const BENCH_DEST_DIR: &str = "./build-hydration-bench";
const DEMO_DEST_DIR: &str = "./build-hydration-demo";

const BENCH_PACKAGE: &str = "vertigo-test-hydration-bench";
const DEMO_PACKAGE: &str = "vertigo-demo";

/// 5555-5559 are taken by `basic`, `reactive_bench`, `dom_bench`, `demo` and the demo API.
///
/// Three servers rather than one, because `disable_hydration` is a per-`MountConfig` flag
/// and `ServerState`'s global registry is keyed by mount point - so each combination needs
/// its own mount point, and a mount point needs its own server.
const SERVERS: [Server; 3] = [
    Server {
        subject: "bench",
        hydration: "on",
        port: 5560,
        mount: "/hb",
        dest_dir: BENCH_DEST_DIR,
        disable_hydration: false,
    },
    Server {
        subject: "bench",
        hydration: "off",
        port: 5561,
        mount: "/hb-off",
        dest_dir: BENCH_DEST_DIR,
        disable_hydration: true,
    },
    Server {
        subject: "demo",
        hydration: "on",
        port: 5562,
        mount: "/demo",
        dest_dir: DEMO_DEST_DIR,
        disable_hydration: false,
    },
];

/// Demo routes free of SSR fetch and websockets.
///
/// Excluded: `/fetch` and `/lazy-list` read a `LazyCache` during render, which issues an SSR
/// fetch; `/chat` and `/ws-collection` are websocket tabs that render only their "turned
/// off" message without the env vars. `/github_explorer` is safe despite the name - it
/// fetches only once a repository has been picked, and none has.
const DEMO_ROUTES: &[&str] = &["/", "/sudoku", "/game_of_life", "/svg"];

/// Discarded. The first load of a route pays for the wasm fetch and Chrome's first
/// compilation of it.
const WARMUP: usize = 2;
/// Timed page loads per row. `VERTIGO_HYDRATION_BENCH_SAMPLES` overrides it.
const SAMPLES: usize = 10;

/// A page that has not hydrated within this is a failure, not a slow sample.
const LOAD_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Copy)]
struct Server {
    subject: &'static str,
    hydration: &'static str,
    port: u16,
    mount: &'static str,
    dest_dir: &'static str,
    disable_hydration: bool,
}

impl Server {
    fn url(&self, route: &str) -> String {
        let route = route.trim_start_matches('/');
        format!("http://127.0.0.1:{}{}/{route}", self.port, self.mount)
    }

    /// Only this suite's own app renders the probe, so only its rows carry marks.
    fn has_probe(&self) -> bool {
        self.subject == "bench"
    }

    fn key(&self, route: &str) -> String {
        key_of(self.subject, route, self.hydration)
    }
}

/// The row key, spelled in exactly one place.
///
/// It is three things at once - the key of the map samples are collected into, the key stored
/// in the JSON and joined on when two runs are compared, and the key the gates look rows up
/// by. Written out separately in each of those places they could drift apart independently,
/// and the failure is silent: the suite would still pass while every row in the comparison
/// came out as "only in this run".
fn key_of(subject: &str, route: &str, hydration: &str) -> String {
    format!("{subject} {route} [{hydration}]")
}

/// One page load's numbers.
#[derive(Debug, Default)]
struct Sample {
    phases: BTreeMap<String, f64>,
    mutations: Mutations,
    coverage: Option<Coverage>,
}

#[tokio::test]
#[ignore]
async fn hydration_bench() -> TestResult {
    // Process-global, which is why this binary holds exactly one test.
    let _ = std::env::set_current_dir("..");

    let samples = std::env::var("VERTIGO_HYDRATION_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(SAMPLES);

    let skip_build = std::env::var("VERTIGO_HYDRATION_BENCH_SKIP_BUILD").is_ok();

    if skip_build {
        println!("Skipping both builds (VERTIGO_HYDRATION_BENCH_SKIP_BUILD)");
    } else {
        build_guest(BENCH_PACKAGE, BENCH_DEST_DIR)?;
        build_guest(DEMO_PACKAGE, DEMO_DEST_DIR)?;
    }

    let mut stoppers = Vec::new();
    for server in SERVERS {
        stoppers.push(spawn_server(server));
    }
    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = ClientBuilder::native()
        .connect("http://localhost:9515")
        .await
        .ctx("failed to connect to WebDriver - is chromedriver running on :9515?")?;

    let user_agent = client
        .execute("return window.navigator.userAgent;", vec![])
        .await
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_default();

    let targets = targets();
    println!(
        "Loading {} pages, {WARMUP} warmup + {samples} timed each",
        targets.len()
    );

    let mut collected: BTreeMap<String, Vec<Sample>> = BTreeMap::new();

    // Round-robin over every row rather than finishing one row at a time, so background load
    // and thermal drift are spread evenly instead of landing on whichever page happened to
    // run while something else was busy.
    for pass in 0..(WARMUP + samples) {
        for (server, route) in &targets {
            let sample = load_once(&client, server, route).await?;

            if pass >= WARMUP {
                collected.entry(server.key(route)).or_default().push(sample);
            }
        }
    }

    client.close().await.ctx("closing the browser failed")?;
    for stopper in stoppers {
        stopper.send(1).ok();
    }
    tokio::time::sleep(Duration::from_millis(500)).await;

    let rows = summarise(&targets, &collected)?;

    let created_at_unix = now_unix();

    let mut meta = Meta::collect();
    meta.user_agent = user_agent;
    // The branch rather than the commit: here the two runs being compared are usually two
    // implementations of hydration living on two branches, not two commits on one.
    let label = meta.label_for(&SUITE, meta.label_branch());

    let run = Run::new(&SUITE, meta, label)
        .harness("warmup", WARMUP)
        .harness("samples", samples)
        .harness("skipped_build", skip_build)
        // Both subjects, because the question this suite exists to answer - what the hydration
        // rewrite cost - is partly a question about wasm size.
        .artifacts(Artifact::scan("bench", BENCH_DEST_DIR)?)
        .artifacts(Artifact::scan("demo", DEMO_DEST_DIR)?)
        .rows(rows);

    run.print();
    print_reading_notes();

    let path = run.write_and_compare(created_at_unix)?;

    assert_rows(&run)?;

    // Last, and only on the way out: `latest.json` is the default baseline, and a run that
    // failed the coverage gates above did not hydrate the way this suite says it must.
    run.promote_to_latest(&path)?;

    Ok(())
}

// --------------------------------------------------------------------------------------
// setup
// --------------------------------------------------------------------------------------

fn targets() -> Vec<(Server, String)> {
    let mut targets = Vec::new();

    for server in SERVERS {
        if server.subject == "bench" {
            for route in Route::ALL {
                targets.push((server, route.path().to_string()));
            }
        } else {
            for route in DEMO_ROUTES {
                targets.push((server, (*route).to_string()));
            }
        }
    }

    targets
}

fn build_guest(package: &str, dest_dir: &str) -> TestResult {
    println!("Building {package}");

    let opts = BuildOpts {
        common: CommonOpts {
            dest_dir: dest_dir.to_string(),
            log_local_time: None,
        },
        inner: build::BuildOptsInner {
            package_name: Some(package.to_string()),
            public_path: None,
            // Both on purpose: the point is to measure the artifact that actually ships.
            wasm_opt: Some(true),
            release_mode: Some(true),
            wasm_run_source_map: false,
            cargo_opts: vec![],
        },
    };

    // `{err:?}` rather than `Ctx`/`Display`: `ErrorCode` implements neither on every branch
    // this suite has to build on, and a benchmark that only compiles against one of the two
    // implementations it compares is not much of a comparison.
    build::run(opts).map_err(|err| format!("building {package} failed: {err:?}").into())
}

fn spawn_server(server: Server) -> oneshot::Sender<i32> {
    let (sender, receiver) = oneshot::channel::<i32>();
    let handle = tokio::runtime::Handle::current();

    println!(
        "Serving {} on port {} at {} (hydration {})",
        server.dest_dir, server.port, server.mount, server.hydration
    );

    std::thread::spawn(move || {
        let opts = ServeOpts {
            common: CommonOpts {
                dest_dir: server.dest_dir.to_string(),
                log_local_time: None,
            },
            inner: ServeOptsInner {
                host: "127.0.0.1".into(),
                port: server.port,
                mount_point: server.mount.to_string(),
                proxy: vec![],
                env: vec![],
                wasm_preload: true,
                disable_hydration: server.disable_hydration,
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
                    if let Err(err) = ret {
                        println!("Can't spawn vertigo-cli on {}: {err:?}", server.port);
                    }
                }
                _ = receiver => {}
            }
        });
    });

    sender
}

// --------------------------------------------------------------------------------------
// one page load
// --------------------------------------------------------------------------------------

/// Everything the page knows about itself, fetched in one round trip once it has settled.
///
/// The sentinel's id arrives as `arguments[0]` rather than being spelled out, so it cannot
/// disagree with [`SENTINEL_ID`] - which the app renders and which the compiler cannot check
/// through a string literal.
const READ_SCRIPT: &str = r#"
    var hb = window.__hb || null;
    var sentinel = document.getElementById(arguments[0]);
    var wasm = performance.getEntriesByType('resource')
        .filter(function (entry) { return entry.name.indexOf('.wasm') !== -1; });

    return {
        probe: hb === null ? null : {
            t_hydrated: hb.t_hydrated,
            mutations: hb.mutations,
            added: hb.added,
            removed: hb.removed,
            attrs: hb.attrs,
            chars: hb.chars
        },
        start: sentinel === null ? null : sentinel.getAttribute('data-start'),
        end: sentinel === null ? null : sentinel.getAttribute('data-end'),
        wasmEnd: wasm.length === 0 ? null : wasm[wasm.length - 1].responseEnd,
        hydration: window.__vertigo_hydration || null
    };
"#;

/// True once the page has finished. For a page carrying the probe that is the sentinel
/// having been observed; for the demo it is the hydration report having been published,
/// which both implementations do at the end of hydration.
const DONE_SCRIPT: &str = r#"
    var probed = arguments[0];
    if (probed) { return !!(window.__hb && window.__hb.done); }
    return !!window.__vertigo_hydration;
"#;

async fn load_once(client: &Client, server: &Server, route: &str) -> TestResult<Sample> {
    let url = server.url(route);

    client
        .goto(&url)
        .await
        .ctx(format!("navigating to {url} failed"))?;

    wait_until_done(client, server, &url).await?;

    let raw = client
        .execute(READ_SCRIPT, vec![serde_json::json!(SENTINEL_ID)])
        .await
        .ctx(format!("reading the page state of {url} failed"))?;

    let number = |value: Option<&serde_json::Value>| -> Option<f64> {
        match value? {
            serde_json::Value::Number(number) => number.as_f64(),
            serde_json::Value::String(text) => text.parse().ok(),
            _ => None,
        }
    };

    let mut sample = Sample::default();

    if let Some(coverage) = raw.get("hydration").filter(|value| !value.is_null()) {
        let field = |name: &str| coverage.get(name).and_then(serde_json::Value::as_u64);

        sample.coverage = Some(Coverage {
            root_found: coverage
                .get("rootFound")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            matched: field("matched").unwrap_or(0),
            hydratable: field("hydratable").unwrap_or(0),
            skipped: field("skipped").unwrap_or(0),
            total: field("total").unwrap_or(0),
        });
    }

    if !server.has_probe() {
        return Ok(sample);
    }

    let probe = raw
        .get("probe")
        .filter(|value| !value.is_null())
        .ctx(format!(
            "{url} published no probe - did the inline script run?"
        ))?;

    let count = |name: &str| {
        probe
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };

    sample.mutations = Mutations {
        mutations: count("mutations"),
        added: count("added"),
        removed: count("removed"),
        attrs: count("attrs"),
        chars: count("chars"),
    };

    let hydrated = number(probe.get("t_hydrated")).ctx(format!("{url}: no t_hydrated"))?;
    let started = number(raw.get("start")).ctx(format!("{url}: sentinel has no data-start"))?;
    let finished = number(raw.get("end")).ctx(format!("{url}: sentinel has no data-end"))?;
    // Fails rather than defaulting to zero, like every other read above it. `wasm_end` is the
    // origin both `boot_ms` and the headline `total_ms` are measured from, so a zero here does
    // not lose a sample - it silently redefines the headline as "time since navigation start"
    // for that one load, inflating it by however long the document took to arrive, and that
    // number then drives the stability check and the whole delta table.
    let wasm_end = number(raw.get("wasmEnd")).ctx(format!(
        "{url}: no resource-timing entry for the wasm, so there is no origin to measure \
         boot_ms and total_ms from"
    ))?;

    sample.phases.insert("boot_ms".into(), started - wasm_end);
    sample.phases.insert("render_ms".into(), finished - started);
    sample
        .phases
        .insert("hydrate_ms".into(), hydrated - finished);
    sample.phases.insert("total_ms".into(), hydrated - wasm_end);

    Ok(sample)
}

/// `Client::wait` has no "until this script is true" form, so poll.
async fn wait_until_done(client: &Client, server: &Server, url: &str) -> TestResult {
    let deadline = std::time::Instant::now() + LOAD_TIMEOUT;
    let probed = serde_json::json!(server.has_probe());

    loop {
        let done = client
            .execute(DONE_SCRIPT, vec![probed.clone()])
            .await
            .ctx(format!("polling {url} failed"))?;

        if done.as_bool().unwrap_or(false) {
            return Ok(());
        }

        if std::time::Instant::now() > deadline {
            return Err(format!(
                "{url} did not hydrate within {LOAD_TIMEOUT:?}. With the probe present that \
                 means #hb-done never appeared; without it, that the framework never \
                 published a hydration report."
            )
            .into());
        }

        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

// --------------------------------------------------------------------------------------
// aggregation
// --------------------------------------------------------------------------------------

fn summarise(
    targets: &[(Server, String)],
    collected: &BTreeMap<String, Vec<Sample>>,
) -> TestResult<Vec<Row>> {
    let mut rows = Vec::new();

    for (server, route) in targets {
        let key = server.key(route);
        let samples = collected
            .get(&key)
            .ctx(format!("no samples were collected for {key}"))?;
        let first = samples
            .first()
            .ctx(format!("no samples were collected for {key}"))?;

        // The counts are a property of the page, not of the run. A route whose mutation
        // count wanders between loads is not being measured, it is being sampled from
        // something non-deterministic, and its timings would mean nothing either.
        for (index, sample) in samples.iter().enumerate() {
            assert_eq!(
                sample.mutations, first.mutations,
                "{key}: load {index} produced different DOM mutation counts from load 0"
            );
            assert_eq!(
                sample.coverage, first.coverage,
                "{key}: load {index} reported different hydration coverage from load 0"
            );
        }

        let mut row = Row::new(&key).samples(samples.len());

        for metric in SUITE.metrics {
            row = row.metric(
                metric.name,
                samples
                    .iter()
                    .filter_map(|sample| sample.phases.get(metric.name).copied())
                    .collect(),
            );
        }

        // Only pages that render the probe have an observer, and only those carry counts. A
        // row without them has to arrive with the keys absent rather than zeroed, or "not
        // measured here" prints as "nothing happened".
        if server.has_probe() {
            let counts = first.mutations;
            row = row
                .counter("mutations", counts.mutations as i64)
                .counter("added", counts.added as i64)
                .counter("removed", counts.removed as i64)
                .counter("attrs", counts.attrs as i64)
                .counter("chars", counts.chars as i64);
        }

        if let Some(coverage) = first.coverage {
            row = row
                .counter("rootFound", i64::from(coverage.root_found))
                .counter("matched", coverage.matched as i64)
                .counter("hydratable", coverage.hydratable as i64)
                .counter("skipped", coverage.skipped as i64)
                .counter("total", coverage.total as i64);
        }

        rows.push(row);
    }

    Ok(rows)
}

/// The two things about this table that are read wrongly if they are not said.
fn print_reading_notes() {
    println!("  render_ms is this app building its own tree - the same work whichever");
    println!("  implementation hydrates it, so it is a control: if it moves between two");
    println!("  runs, something other than hydration did. hydrate_ms is the answer.");
    println!();
    println!("  mutations counts operations on nodes already in the document, not nodes");
    println!("  touched: attaching a subtree is one record whatever its size. So the [off]");
    println!("  rows, which build detached and attach once, are not a mutation baseline for");
    println!("  the [on] rows and nothing here compares them. Between two hydration");
    println!("  implementations the count is exactly comparable - both work on the same");
    println!("  attached server nodes.");
    println!();
    println!("  A `-` is a row that carries no probe: the demo cannot render one, so its rows");
    println!("  carry hydration coverage only.");
    println!();
}

// --------------------------------------------------------------------------------------
// the gates
// --------------------------------------------------------------------------------------

/// The hydration report a row carried, or `None` when none was published.
///
/// Reconstructed from the counters rather than kept beside them: absence is the signal the
/// `[off]` rows are checked on, and a counter map records absence natively.
fn coverage_of(row: &Row) -> Option<Coverage> {
    row.counters.contains_key("hydratable").then(|| Coverage {
        root_found: row.get_counter("rootFound") != 0,
        matched: row.get_counter("matched") as u64,
        hydratable: row.get_counter("hydratable") as u64,
        skipped: row.get_counter("skipped") as u64,
        total: row.get_counter("total") as u64,
    })
}

/// `hydratable - matched`, signed.
///
/// Read only inside an assertion message that has already failed - which is precisely the case
/// where `matched` may be the larger of the two. This suite runs `--release`, where overflow
/// checks are off, so an unsigned subtraction would report eighteen quintillion unmatched nodes
/// instead of saying that more were matched than were hydratable.
fn unmatched(coverage: &Coverage) -> i64 {
    coverage.hydratable as i64 - coverage.matched as i64
}

fn assert_rows(run: &Run) -> TestResult {
    let find = |subject: &str, route: &str, hydration: &str| -> TestResult<&Row> {
        let key = key_of(subject, route, hydration);
        run.rows
            .iter()
            .find(|row| row.key == key)
            .ctx(format!("{key} is missing from the report"))
    };

    for route in Route::ALL {
        let on = find("bench", route.path(), "on")?;
        let off = find("bench", route.path(), "off")?;

        let coverage = coverage_of(on).ctx(format!("{} published no hydration report", on.key))?;

        assert!(
            coverage.root_found,
            "{}: hydration never found <body>, so the server markup was replaced wholesale \
             rather than adopted - {coverage:?}",
            on.key
        );
        assert!(
            coverage.hydratable > 0,
            "{}: nothing was hydratable, so this row proves nothing - {coverage:?}",
            on.key
        );

        // Neither implementation publishes a report when hydration is off. If one appears
        // here, the control is not a control and every comparison against it is void.
        assert!(
            coverage_of(off).is_none(),
            "{}: a hydration report was published although the server was started with \
             --disable-hydration",
            off.key
        );

        // The sentinel is in the server's markup carrying zeros and in the browser's carrying
        // real marks, so whichever implementation hydrates it has to write both attributes
        // onto a node that is already in the document. Two attribute mutations is therefore
        // the floor for any hydrated page, on any implementation - and unlike a node count it
        // cannot be hidden by building a subtree detached. If this is zero the observer is not
        // seeing the document and every number in the run is worthless.
        assert!(
            on.get_counter("attrs") >= 2,
            "{}: the observer recorded {} attribute mutations. Hydrating the sentinel has to \
             produce at least two, so the probe is not seeing what it should",
            on.key,
            on.get_counter("attrs"),
        );
    }

    // The clean-match pages are the ones whose server and browser trees agree, so every
    // hydratable node should have been adopted. This is the check that says hydration
    // *happened*, on either implementation.
    for route in Route::CLEAN {
        let row = find("bench", route.path(), "on")?;
        let coverage =
            coverage_of(row).ctx(format!("{} published no hydration report", row.key))?;

        assert_eq!(
            coverage.matched,
            coverage.hydratable,
            "{}: {} of {} nodes went unmatched, so that much of the server-rendered page was \
             rebuilt - {coverage:?}",
            row.key,
            unmatched(&coverage),
            coverage.hydratable,
        );
    }

    for route in DEMO_ROUTES {
        let row = find("demo", route, "on")?;
        let coverage =
            coverage_of(row).ctx(format!("{} published no hydration report", row.key))?;

        assert!(
            coverage.root_found && coverage.hydratable > 0,
            "{}: the demo did not hydrate - {coverage:?}",
            row.key
        );
        assert_eq!(
            coverage.matched,
            coverage.hydratable,
            "{}: {} of {} nodes went unmatched in the demo - {coverage:?}",
            row.key,
            unmatched(&coverage),
            coverage.hydratable,
        );
    }

    Ok(())
}
