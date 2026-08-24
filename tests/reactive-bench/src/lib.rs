//! Reactive-graph benchmark, run as WASM in a real browser.
//!
//! Driven by `tests/reactive-bench/tests.rs` over WebDriver. The app runs the workloads in
//! [`workloads::WORKLOADS`], writes the results into the DOM, and finally renders a
//! `#bench-done` sentinel for the test to poll on.
//!
//! Why this exists: the graph's recent optimisations were allocation-shaped, and wasm32 uses
//! dlmalloc where native builds use glibc malloc. The native harness in
//! `crates/vertigo/src/reactive_old/compare.rs` cannot see that difference.

use vertigo::{DomElement, DomNode, Value, dom, get_driver, main};

mod clock;
mod runner;
mod workloads;

use clock::{now_ms, read_scale, user_agent};
use runner::{Row, report_text, run_one};
use workloads::WORKLOADS;

#[derive(Clone, PartialEq)]
enum State {
    /// What the server renders, and what the browser renders first - identical on purpose,
    /// so hydration has nothing to reconcile.
    Pending,
    Running {
        done: Vec<Row>,
        current: &'static str,
    },
    Finished {
        rows: Vec<Row>,
        total_ms: f64,
        user_agent: String,
    },
}

#[main]
fn render() -> DomNode {
    let state = Value::new(State::Pending);

    // `vertigo-cli serve` runs this same entry function server-side to produce the initial
    // HTML, and answers `IsBrowser` with false. Without this gate every page request would
    // run the whole benchmark inside the server's wasm instance.
    if get_driver().is_browser() {
        spawn_run(state.clone());
    }

    dom! {
        <html>
            <head />
            <body>
                {render_body(state)}
            </body>
        </html>
    }
}

/// Deferred rather than run inline in the entry function.
///
/// `start_app` calls the entry function *before* `set_root`, so running the workloads inline
/// would block before the first paint, make the browser's first tree differ from the
/// server's, and leave a hang indistinguishable from a slow load. Deferring keeps the first
/// render identical to the SSR one, publishes progress, and yields to the event loop between
/// workloads so WebDriver stays responsive.
fn spawn_run(state: Value<State>) {
    vertigo::spawn(async move {
        let driver = get_driver();

        // Let the first paint and hydration land before blocking the main thread.
        driver.sleep(50).await;

        let scale = read_scale();
        let started = now_ms();
        let mut rows: Vec<Row> = Vec::new();

        for workload in WORKLOADS {
            state.set(State::Running {
                done: rows.clone(),
                current: workload.slug,
            });
            // Flush the progress update and hand the event loop back before the next
            // hundreds of milliseconds of blocking work.
            driver.sleep(0).await;

            rows.push(run_one(workload, scale));
        }

        state.set(State::Finished {
            total_ms: now_ms() - started,
            rows,
            user_agent: user_agent(),
        });
    });
}

/// Everything comes out of one `Value<State>` through one `render_value`, which matters:
/// a single `set` produces one propagation wave and one bulk DOM update applied in one JS
/// callback. WebDriver therefore cannot observe `#bench-done` while `#bench-report` is still
/// stale. Splitting the sentinel into its own `Value` would reintroduce that race.
fn render_body(state: Value<State>) -> DomNode {
    state.render_value(|state| match state {
        State::Pending => dom! {
            <div>
                <div id="bench-status">"pending"</div>
            </div>
        },
        State::Running { done, current } => dom! {
            <div>
                <div id="bench-status">"running"</div>
                <div id="bench-current">{current}</div>
                {rows_table(done)}
            </div>
        },
        State::Finished {
            rows,
            total_ms,
            user_agent,
        } => {
            let report = report_text(&rows);
            dom! {
                <div>
                    <div id="bench-status">"finished"</div>
                    <div id="bench-ua">{user_agent}</div>
                    <div id="bench-total-ms">{format!("{total_ms:.1}")}</div>
                    {rows_table(rows)}
                    <pre id="bench-report">{report}</pre>
                    // Only reachable behind the `is_browser` gate, so server-rendered HTML
                    // provably never contains it. The test polls for exactly this.
                    <div id="bench-done">"ok"</div>
                </div>
            }
        }
    })
}

fn rows_table(rows: Vec<Row>) -> DomNode {
    let children = rows
        .into_iter()
        .map(|row| {
            dom! {
                <div id={format!("bench-{}", row.slug)}>
                    <span>{row.title}</span>
                    <span id={format!("bench-{}-per-op-us", row.slug)}>
                        {format!("{:.4}", row.per_op_us())}
                    </span>
                    <span id={format!("bench-{}-best-ms", row.slug)}>
                        {format!("{:.3}", row.best_ms)}
                    </span>
                    <span id={format!("bench-{}-iters", row.slug)}>{row.iters}</span>
                    <span id={format!("bench-{}-runs", row.slug)}>{row.runs}</span>
                </div>
            }
        })
        .collect::<Vec<DomNode>>();

    DomElement::new("div")
        .attr("id", "bench-rows")
        .children(children)
        .into()
}
