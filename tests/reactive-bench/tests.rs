//! Runs the reactive-graph benchmark in a real browser and prints the results.
//!
//! Requires a WebDriver on localhost:9515. Run with:
//!
//! ```text
//! cargo test --package fantoccini-tests --test reactive_bench -- --ignored --nocapture
//! ```
//!
//! This is a *reporter*, not a perf gate: it asserts only on things that hold regardless of
//! machine speed (see `assert_row` and the cutoff/fan-out invariants at the bottom).

use std::time::Duration;

use fantoccini::{Client, ClientBuilder, Locator};
use vertigo_cli::{BuildOpts, CommonOpts, ServeOpts, build, serve};

/// Must differ from `basic` (5555): cargo may run the two test binaries concurrently.
const PORT: u16 = 5556;
/// Must also differ - `build::run` starts by wiping its dest dir, so sharing `./build`
/// would let one test delete the other's artifacts.
const DEST_DIR: &str = "./build-reactive-bench";
const PACKAGE: &str = "vertigo-test-reactive-bench";
const RUN_TIMEOUT: Duration = Duration::from_secs(300);

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
    median_ms: f64,
    per_op_us: f64,
    runs: u64,
    checksum: u64,
}

#[tokio::test]
#[ignore]
async fn reactive_bench() {
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
        .expect("failed to connect to WebDriver - is chromedriver running on :9515?");

    let site_url = format!("http://127.0.0.1:{PORT}/");
    println!("Opening {site_url}");
    client.goto(&site_url).await.expect("goto failed");

    println!("Waiting for the benchmark to finish (timeout {RUN_TIMEOUT:?})");
    wait_for_done(&client, RUN_TIMEOUT).await;

    let report = text_of(&client, "bench-report")
        .await
        .expect("#bench-report missing");
    let user_agent = text_of(&client, "bench-ua").await.unwrap_or_default();
    let total_ms = text_of(&client, "bench-total-ms").await.unwrap_or_default();

    let rows = parse_report(&report);

    print_table(&rows, &user_agent, &total_ms);

    client.close().await.expect("close failed");
    sender.send(1).ok();
    tokio::time::sleep(Duration::from_secs(1)).await;

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
}

/// `Client::find` does not retry, so poll - and on timeout say what the page was doing,
/// rather than just that an element was missing.
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
                7,
                "malformed report line {line:?} - expected 7 `|`-separated fields"
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
                median_ms: number(3),
                per_op_us: number(4),
                runs: number(5) as u64,
                checksum: number(6) as u64,
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

fn print_table(rows: &[Reported], user_agent: &str, total_ms: &str) {
    println!();
    println!("reactive graph, WASM in a browser");
    println!("  user agent : {user_agent}");
    println!("  total      : {total_ms} ms");
    println!();
    println!(
        "  {:<16} {:>10} {:>12} {:>12} {:>14} {:>12}",
        "workload", "iters", "best (ms)", "median (ms)", "per op (us)", "runs"
    );
    for row in rows {
        println!(
            "  {:<16} {:>10} {:>12.3} {:>12.3} {:>14.4} {:>12}",
            row.slug, row.iters, row.best_ms, row.median_ms, row.per_op_us, row.runs
        );
    }
    println!();
    // Printed so a pasted table cannot be mistaken for one whose work was optimised away.
    let checksums: Vec<String> = rows
        .iter()
        .map(|row| format!("{}={}", row.slug, row.checksum))
        .collect();
    println!("  checksums  : {}", checksums.join(" "));
    println!();
}
