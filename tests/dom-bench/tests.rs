//! Runs the rendering / DOM benchmark in a real browser and prints the results.
//!
//! Requires a WebDriver on localhost:9515. Run with:
//!
//! ```text
//! cargo test --package fantoccini-tests --test dom_bench -- --ignored --nocapture
//! ```
//!
//! Timings are printed, never asserted - they swing several-fold across machines. What *is*
//! asserted is the DOM command count per operation, which is deterministic and is the real
//! regression guard for the list reconciler and the text-update paths.

use std::{collections::BTreeMap, time::Duration};

use fantoccini::{Client, ClientBuilder, Locator};
use vertigo_cli::{BuildOpts, CommonOpts, ServeOpts, build, serve};

/// Distinct from `basic` (5555) and `reactive_bench` (5556): cargo may run the three test
/// binaries concurrently.
const PORT: u16 = 5557;
/// Distinct for the same reason, and because `build::run` wipes its dest dir on entry.
const DEST_DIR: &str = "./build-dom-bench";
const PACKAGE: &str = "vertigo-test-dom-bench";
const RUN_TIMEOUT: Duration = Duration::from_secs(300);

/// Mirrors the scene constants in the app crate. Kept in step by the assertions below,
/// which fail loudly if the app changes and this does not.
const ITEMS: u32 = 500;
const SITES: u64 = 200;

/// Every workload expected in the report, so one silently dropped from the table fails.
const EXPECTED_SLUGS: &[&str] = &[
    "list-mount-unmount",
    "list-append-remove",
    "list-append-remove-small",
    "list-middle-remove-reinsert",
    "list-reverse",
    "list-reverse-layout",
    "list-update-text",
    "list-toggle-class",
    "editor-keystroke-embed",
    "editor-keystroke-patch",
    "editor-toggle-bold",
    "editor-caret-move",
    "editor-block-insert-delete",
    "dash-tick-all",
    "dash-tick-one",
    "dash-status-change",
    "flush-min",
];

#[derive(Debug, Clone)]
struct Reported {
    slug: String,
    iters: u64,
    best_ms: f64,
    median_ms: f64,
    per_op_us: f64,
    runs: u64,
    checksum: u64,
    cmds: u32,
    leaked: i64,
    breakdown: BTreeMap<String, u32>,
}

impl Reported {
    /// Count for one command variant, zero when the operation never emits it.
    fn cmd(&self, name: &str) -> u32 {
        self.breakdown.get(name).copied().unwrap_or(0)
    }
}

#[tokio::test]
#[ignore]
async fn dom_bench() {
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
            // Both on purpose. Release also matters for correctness of the counts: debug
            // builds emit extra `v-component` and `v-css` attribute commands per instance.
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

    // --- assertions: deterministic only -------------------------------------

    for slug in EXPECTED_SLUGS {
        assert!(
            rows.iter().any(|row| row.slug == *slug),
            "workload {slug} missing from the report"
        );
    }

    for row in &rows {
        assert_row(row);
    }

    assert_writes(&rows);
    assert_text_paths(&rows);
    assert_attribute_paths(&rows);
    assert_list_reconciler(&rows);
    assert_dashboard(&rows);
    assert_paired_operations_balance(&rows);

    for row in &rows {
        assert_eq!(
            row.leaked, 0,
            "{}: left {} tracked DOM nodes behind after teardown",
            row.slug, row.leaked
        );
    }
}

/// Writes performed per operation. This is what makes "one operation is a pair" checkable
/// from the report rather than only from the source.
const WRITES_PER_OP: &[(&str, u64)] = &[
    ("list-mount-unmount", 2),
    ("list-append-remove", 2),
    ("list-append-remove-small", 2),
    ("list-middle-remove-reinsert", 2),
    ("list-reverse", 1),
    ("list-reverse-layout", 1),
    ("list-update-text", 1),
    ("list-toggle-class", 1),
    ("editor-keystroke-embed", 1),
    ("editor-keystroke-patch", 1),
    ("editor-toggle-bold", 1),
    ("editor-caret-move", 1),
    ("editor-block-insert-delete", 2),
    ("dash-tick-all", SITES),
    ("dash-tick-one", 1),
    ("dash-status-change", 2),
    ("flush-min", 1),
];

fn assert_writes(rows: &[Reported]) {
    for (slug, per_op) in WRITES_PER_OP {
        let row = find_row(rows, slug);
        assert_eq!(
            row.runs,
            row.iters * per_op,
            "{slug}: expected {per_op} write(s) per operation over {} iterations",
            row.iters
        );
    }
}

/// The two ways a `Value<String>` can reach a text node must cost the same.
///
/// `{value}` interpolation used to wrap the text in a `render_value`, which replaced the
/// whole node on every change: three commands against one, plus a marker comment per
/// interpolation and a fresh id each time. It now patches, like the hand-written form. This
/// pair is what stops that regressing - the two workloads are the same scene built with the
/// two spellings, so any divergence shows up here.
fn assert_text_paths(rows: &[Reported]) {
    let embed = find_row(rows, "editor-keystroke-embed");
    let patch = find_row(rows, "editor-keystroke-patch");

    for row in [embed, patch] {
        assert_eq!(row.cmds, 1, "{}: {:?}", row.slug, row.breakdown);
        assert_eq!(row.cmd("UpdateText"), 1, "{}: and it is a patch", row.slug);
    }
    assert_eq!(
        embed.breakdown, patch.breakdown,
        "ordinary interpolation must take the same path as DomText::new_computed"
    );

    // Same mechanism again, in a different scene; catches the two drifting apart.
    let list_text = find_row(rows, "list-update-text");
    assert_eq!(
        list_text.breakdown, embed.breakdown,
        "a list label update and an editor keystroke take the same path"
    );

    // A caret move inside one formatting run propagates through the graph but is stopped by
    // the equality cutoff before it reaches the DOM. Read together with the write count
    // asserted above, which is what distinguishes this from the work being optimised away.
    let caret = find_row(rows, "editor-caret-move");
    assert_eq!(
        caret.cmds, 0,
        "a caret move inside one run must emit no DOM commands, got {:?}",
        caret.breakdown
    );
}

fn assert_attribute_paths(rows: &[Reported]) {
    for slug in ["list-toggle-class", "editor-toggle-bold", "flush-min"] {
        let row = find_row(rows, slug);
        assert_eq!(row.cmds, 1, "{slug}: {:?}", row.breakdown);
        assert_eq!(
            row.cmd("SetAttr"),
            1,
            "{slug}: and it is an attribute write"
        );
    }
}

fn assert_list_reconciler(rows: &[Reported]) {
    let append = find_row(rows, "list-append-remove");
    let small = find_row(rows, "list-append-remove-small");
    let middle = find_row(rows, "list-middle-remove-reinsert");

    // Position does not change the cost: the prefix/suffix phases and the keyed middle map
    // must produce the same commands wherever the row churns.
    assert_eq!(
        append.breakdown, middle.breakdown,
        "appending and re-inserting mid-list must cost the same commands"
    );
    // Nor does list length.
    assert_eq!(
        append.breakdown, small.breakdown,
        "one row's churn must not depend on how many rows surround it"
    );

    // A full reverse is the reconciler's worst case: prefix and suffix match nothing, so
    // every row goes through the middle map. Rows are moved, never rebuilt - two
    // `InsertBefore` each, for the row's anchor comment and its content.
    let reverse = find_row(rows, "list-reverse");
    assert_eq!(
        reverse.cmd("InsertBefore"),
        2 * ITEMS,
        "reversing {ITEMS} rows re-inserts each row's anchor and content"
    );
    assert_eq!(
        reverse.cmds,
        2 * ITEMS,
        "and does nothing else - no row is rebuilt: {:?}",
        reverse.breakdown
    );
}

fn assert_dashboard(rows: &[Reported]) {
    let one = find_row(rows, "dash-tick-one");
    let all = find_row(rows, "dash-tick-all");

    // The aggregate depends on the status flag, not the latency, so a tick is exactly one
    // row's text update - and a text update is one patch.
    assert_eq!(one.cmds, 1, "dash-tick-one: {:?}", one.breakdown);
    assert_eq!(one.cmd("UpdateText"), 1);

    // One transaction covering every site costs exactly what the sites cost individually -
    // the batching saves flushes, not commands.
    assert_eq!(
        all.cmds,
        SITES as u32 * one.cmds,
        "ticking all {SITES} sites in one transaction: {:?}",
        all.breakdown
    );
}

/// Every paired operation ends where it started, so whatever it created it must also have
/// removed. Self-checking, and it needs no knowledge of the per-row constant.
fn assert_paired_operations_balance(rows: &[Reported]) {
    const PAIRED: &[&str] = &[
        "list-mount-unmount",
        "list-append-remove",
        "list-append-remove-small",
        "list-middle-remove-reinsert",
        "editor-block-insert-delete",
        "dash-status-change",
    ];

    for slug in PAIRED {
        let row = find_row(rows, slug);
        let created = row.cmd("CreateNode") + row.cmd("CreateText") + row.cmd("CreateComment");
        let removed = row.cmd("RemoveNode") + row.cmd("RemoveText") + row.cmd("RemoveComment");
        assert!(created > 0, "{slug}: created nothing, so it is not a pair");
        assert_eq!(
            created, removed,
            "{slug}: created {created} nodes but removed {removed}: {:?}",
            row.breakdown
        );
    }
}

/// `Client::find` does not retry, so use the waiter - and on timeout say what the page was
/// doing rather than just that an element was missing.
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
        // A missing element is an answer; anything else means the session is in trouble.
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
                10,
                "malformed report line {line:?} - expected 10 `|`-separated fields"
            );
            let number = |index: usize| -> f64 {
                fields[index]
                    .parse()
                    .unwrap_or_else(|_| panic!("field {index} of {line:?} is not a number"))
            };
            // `-` rather than an empty field, so the field count never depends on the data.
            let breakdown = if fields[9] == "-" {
                BTreeMap::new()
            } else {
                fields[9]
                    .split(',')
                    .map(|pair| {
                        let (name, count) = pair
                            .split_once('=')
                            .unwrap_or_else(|| panic!("malformed breakdown {pair:?}"));
                        let count: u32 = count
                            .parse()
                            .unwrap_or_else(|_| panic!("malformed breakdown count {pair:?}"));
                        (name.to_string(), count)
                    })
                    .collect()
            };

            Reported {
                slug: fields[0].to_string(),
                iters: number(1) as u64,
                best_ms: number(2),
                median_ms: number(3),
                per_op_us: number(4),
                runs: number(5) as u64,
                checksum: number(6) as u64,
                cmds: number(7) as u32,
                leaked: number(8) as i64,
                breakdown,
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
        runs,
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
    // Distinguishes "the write was cut off before the DOM" from "the write was optimised
    // away", which look identical if you only read the command count.
    assert!(*runs > 0, "{slug}: no writes were performed");
}

fn print_table(rows: &[Reported], user_agent: &str, total_ms: &str) {
    println!();
    println!("rendering / DOM operations, WASM in a browser");
    println!("  user agent : {user_agent}");
    println!("  total      : {total_ms} ms");
    println!();
    println!(
        "  {:<28} {:>9} {:>11} {:>11} {:>13} {:>6} {:>7}",
        "workload", "iters", "best (ms)", "med (ms)", "per op (us)", "cmds", "leaked"
    );
    for row in rows {
        // Median alongside best: a median far above the best means the run was disturbed,
        // and the per-op figure should not be trusted without a re-run.
        println!(
            "  {:<28} {:>9} {:>11.3} {:>11.3} {:>13.4} {:>6} {:>7}",
            row.slug, row.iters, row.best_ms, row.median_ms, row.per_op_us, row.cmds, row.leaked
        );
    }
    println!();
    println!("  DOM commands per operation:");
    for row in rows {
        let breakdown = if row.breakdown.is_empty() {
            "(none)".to_string()
        } else {
            row.breakdown
                .iter()
                .map(|(name, count)| format!("{name}={count}"))
                .collect::<Vec<_>>()
                .join(" ")
        };
        println!("  {:<28} {breakdown}", row.slug);
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
