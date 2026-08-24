//! Rendering and DOM-operation benchmark, run as WASM in a real browser.
//!
//! Driven by `tests/dom-bench/tests.rs` over WebDriver. Three app shapes - a list widget, a
//! text editor, a statistics dashboard - are mounted into a stage element and mutated, and
//! both the wall-clock cost and the exact DOM commands each operation emits are reported.
//!
//! The companion suite `tests/reactive-bench` measures the reactive graph with the DOM
//! deliberately excluded. This is the other half: what an app actually spends creating,
//! patching and reordering nodes.
//!
//! Why timing a plain `Value::set` is enough to capture all of it: the driver registers
//! `on_after_transaction(|| flush_dom_changes())`, and the JS side applies every command
//! inline - no rAF, no microtask batching. One `set` therefore runs propagation, command
//! generation, serialization, the wasm to JS call, and the real `createElement` /
//! `insertBefore` / `setAttribute` calls, before it returns. Browser *layout* is still
//! deferred, which is what the `-layout` workload variant exists to expose.

use vertigo::{DomElement, DomNode, Value, dom, get_driver, main};

mod scenes;
mod stage;
mod workloads;

use stage::stage_node;
use vertigo_bench_support::{Row, RunOpts, now_ms, read_scale, report_text, run_one, user_agent};
use workloads::WORKLOADS;

#[derive(Clone, PartialEq)]
enum State {
    /// What the server renders, and what the browser renders first - identical on purpose,
    /// so hydration has nothing to reconcile.
    Pending,
    Running {
        current: &'static str,
        done: usize,
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
                // A sibling of the progress UI, never a child: the progress `render_value`
                // rebuilds its subtree on every update, and the stage must survive that.
                {stage_node()}
            </body>
        </html>
    }
}

/// Deferred rather than run inline in the entry function.
///
/// `start_app` calls the entry function *before* `set_root`, so running inline would block
/// before the first paint, make the browser's first tree differ from the server's, and leave
/// a hang indistinguishable from a slow load.
fn spawn_run(state: Value<State>) {
    vertigo::spawn(async move {
        let driver = get_driver();

        // Let the first paint and hydration land before blocking the main thread. Hydration
        // only ever touches that first flush, long before any workload mounts a scene.
        driver.sleep(50).await;

        let scale = read_scale();
        let started = now_ms();
        let mut rows: Vec<Row> = Vec::new();

        let opts = RunOpts {
            count_commands: true,
            node_census: true,
        };

        for workload in WORKLOADS {
            state.set(State::Running {
                current: workload.slug,
                done: rows.len(),
            });
            // Flush the progress update and hand the event loop back before the next
            // hundreds of milliseconds of blocking work.
            driver.sleep(0).await;

            rows.push(run_one(workload, scale, opts));
        }

        state.set(State::Finished {
            total_ms: now_ms() - started,
            rows,
            user_agent: user_agent(),
        });
    });
}

/// Everything comes out of one `Value<State>` through one `render_value`: a single `set`
/// produces one propagation wave and one bulk DOM update applied in one JS callback, so
/// WebDriver cannot observe `#bench-done` while `#bench-report` is still stale.
///
/// The results table is rendered only in `Finished`. Rendering it on every progress update
/// would rebuild a growing table once per workload, inside the same document the stage
/// lives in.
fn render_body(state: Value<State>) -> DomNode {
    state.render_value(|state| match state {
        State::Pending => dom! {
            <div>
                <div id="bench-status">"pending"</div>
            </div>
        },
        State::Running { current, done } => dom! {
            <div>
                <div id="bench-status">"running"</div>
                <div id="bench-current">{current}</div>
                <div id="bench-done-count">{done}</div>
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
            let cmds = row.cmds.as_ref().map(|cmds| cmds.total).unwrap_or(0);
            dom! {
                <div id={format!("bench-{}", row.slug)}>
                    <span>{row.title}</span>
                    <span id={format!("bench-{}-per-op-us", row.slug)}>
                        {format!("{:.4}", row.per_op_us())}
                    </span>
                    <span id={format!("bench-{}-cmds", row.slug)}>{cmds}</span>
                </div>
            }
        })
        .collect::<Vec<DomNode>>();

    DomElement::new("div")
        .attr("id", "bench-rows")
        .children(children)
        .into()
}
