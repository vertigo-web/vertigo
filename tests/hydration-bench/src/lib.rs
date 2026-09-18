//! Hydration benchmark subject, run as WASM in a real browser.
//!
//! Driven by `tests/hydration-bench/tests.rs`. Unlike the SSR benchmark this one needs a
//! browser: hydration is a DOM operation, and the thing being measured is how much of a
//! server-rendered document survives contact with the application.
//!
//! ## Why this app is written the way it is
//!
//! The same benchmark has to run against two implementations that live on two branches -
//! hydration in JavaScript, and hydration rewritten in Rust - so it may not depend on
//! anything either of them added. It therefore measures through three things that predate
//! both and are identical on both:
//!
//! - `window.__vertigo_hydration`, whose five field names are an older contract than either
//!   implementation;
//! - a `MutationObserver`, which counts what the browser's DOM actually had done to it and
//!   does not care where the matching ran;
//! - `performance.now()`, read by this app through `js!` at two points it controls.
//!
//! Nothing in `crates/vertigo` is modified for this, on either branch. That is deliberate:
//! instrumentation added to two implementations separately is a thing the comparison could
//! be an artifact of.
//!
//! ## The marks
//!
//! `#hb-done` carries `data-start` / `data-end` - `performance.now()` read at the first and
//! last line of the render below, and zero on the server. One element does both jobs: its
//! `data-end` going non-zero is the signal that the batch has been fully applied (the
//! probe's observer watches for it), and it is the carrier for the two marks, so no extra
//! channel to the page is needed.
//!
//! - `render_ms` = `data-end - data-start`: this app building its own tree. The same work on
//!   both implementations, so if it differs between two runs, something other than hydration
//!   moved and the rest of the table should not be read.
//! - `hydrate_ms` = `t_hydrated - data-end`: everything the framework does with that tree.
//!   This is the number the benchmark exists to produce.
//!
//! ## What this app must never do
//!
//! No `fetch`, no websockets, no zero-delay timers, and nothing from the clock or the RNG in
//! the rendered markup - every load of a route must produce byte-identical HTML, or repeated
//! samples are not samples of the same thing.

use vertigo::{DomElement, DomNode, JsJson, js, main, router::Router, transaction};

pub mod pages;
pub mod probe;
pub mod route;
pub mod shapes;

use probe::probe_js;
use route::Route;

/// Where the probe parks its marks, and the id the driver reads them back from.
pub const SENTINEL_ID: &str = "hb-done";

#[main]
fn render() -> DomNode {
    let started = now_ms();

    // Read, never subscribed to: a subscription would run `Value::with_connect` and register
    // a location callback, adding a boundary round trip to every page for nothing.
    let route = transaction(|context| Router::<Route>::new_history_router().route.get(context));

    let body = match route {
        Route::Tiny | Route::NotFound => pages::tiny(),
        Route::Wide => pages::wide(),
        Route::Deep => pages::deep(),
        Route::Text => pages::text(),
        Route::Attrs => pages::attrs(),
        Route::Table => pages::table(),
        Route::MismatchOrder => pages::mismatch_order(),
        Route::MismatchDepth => pages::mismatch_depth(),
        Route::MismatchAttrs => pages::mismatch_attrs(),
        Route::MismatchText => pages::mismatch_text(),
    };

    shell(body, started)
}

/// `performance.now()`, or 0 on the server where a JS call answers `Null`.
fn now_ms() -> f64 {
    match js! { window.performance.now() } {
        JsJson::Number(number) => number.as_f64(),
        _ => 0.0,
    }
}

fn shell(body: DomNode, started: f64) -> DomNode {
    let head = DomElement::new("head")
        .child(DomElement::new("title").child_text("hydration-bench"))
        // Server-rendered, so it runs during parsing - before wasm, and before anything this
        // benchmark wants to measure.
        .child(DomElement::new("script").child_text(probe_js()));

    let page = DomElement::new("div").attr("id", "rows").child(body);
    let body_element = DomElement::new("body").child(page);

    // Rendered on the server too, carrying zeros - `js!` answers `Null` off-browser, so
    // `now_ms` is 0.0 there.
    //
    // Deliberately not browser-only. A node the server never sent cannot be matched by
    // anything, so it would sit in `hydratable` for ever unmatched and the natural invariant
    // "`matched == hydratable` on a page that agrees with itself" could never be written -
    // every clean page came out exactly one short. Rendering it on both sides keeps the two
    // trees the same shape; the browser's two attribute values differ, which is what the
    // probe waits for and costs two attribute mutations.
    //
    // Built last, so `data-end` covers the whole of the render above it.
    let finished = now_ms();
    body_element.add_child(
        DomElement::new("div")
            .attr("id", SENTINEL_ID)
            .attr("hidden", "hidden")
            .attr("data-start", format!("{started:.4}"))
            .attr("data-end", format!("{finished:.4}")),
    );

    DomElement::new("html")
        .child(head)
        .child(body_element)
        .into()
}
