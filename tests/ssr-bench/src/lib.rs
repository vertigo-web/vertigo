//! Server-side rendering benchmark, run natively on the host.
//!
//! Driven by `tests/ssr-bench/tests.rs`, which does **not** start a browser or a web
//! server: SSR is wasmtime plus `vertigo-cli`, both of which run on the host, so the driver
//! calls `ServerState::request_timed` in-process and reads the phase breakdown straight
//! out.
//!
//! The companion browser suites measure the other side of the same framework:
//! `tests/reactive-bench` the graph alone, `tests/dom-bench` real rendering into a real DOM.
//!
//! ## What each page is for
//!
//! Every route renders the same shell and wrapper as `/`, plus N copies of exactly one
//! thing. Results are read as `t(page) − t(/)`, which cancels instantiation and the fixed
//! shell without anyone needing to know what either costs - and makes the command-count
//! assertions in `tests.rs` a clean `N * per_unit` with no shell arithmetic in them.
//!
//! Two of the pages come in pairs that differ in one respect only, so the difference
//! between them prices that one thing: `/deep` against `/deep-indent` is the
//! pretty-printer, `/text` against `/text-plain` is HTML escaping.
//!
//! ## What this app must never do
//!
//! - **No `fetch`.** An SSR fetch is issued with `actix_web::rt::spawn`, which is
//!   `spawn_local` and panics outside a `LocalSet` - and the render loop then parks waiting
//!   for the network, which is not what any of these phases are measuring.
//! - **No websockets.** The host answers those commands with `Null` and drops them, so the
//!   page would render nothing.
//! - **No zero-delay timers.** `TimerSet { duration: 0 }` re-enters wasm from the drain
//!   loop, adding calls that vary with scheduling. A non-zero timer is discarded by the
//!   host, so it would measure nothing either.
//! - **Nothing from the clock or the RNG in the output.** The driver asserts that every
//!   render of a route produces a byte-identical body; that assertion is what proves no
//!   guest state leaks between requests, and a timestamp in the markup would destroy it.

use vertigo::{DomElement, DomNode, get_driver, main, router::Router, transaction};

// Public because the driver depends on this crate as an rlib and reads `Route::ALL` and the
// size constants straight out of it. `dom-bench` duplicates its scene sizes into its driver
// and relies on assertions to catch the drift; there is no need for that here, because
// unlike a browser suite the driver and the app can share a compilation.
pub mod pages;
pub mod route;
pub mod shapes;

use route::Route;
use shapes::PLAIN_BODY;

#[main]
fn render() -> DomNode {
    // Registered before the tree is built, but answered by `vertigo_export_handle_url`
    // *after* the mount has already run and already shipped its DOM batch. That ordering is
    // the whole point of `/plain.txt`: it is the one route where phases 1-3 happen and
    // phase 4 does not.
    get_driver().plains(|url| (url == "/plain.txt").then(|| PLAIN_BODY.to_string()));

    // Read inside a transaction rather than through `render_value`, so the route is *read*
    // and never *subscribed to*. A subscription would run `Value::with_connect`, which
    // registers a location callback - one more host round trip on every route, measuring
    // the harness instead of the page.
    let route = transaction(|context| Router::<Route>::new_history_router().route.get(context));

    let body = match route {
        Route::Tiny | Route::NotFound => pages::tiny(),
        Route::Wide => pages::wide(),
        Route::Deep => pages::deep(),
        Route::DeepIndent => pages::deep_indent(),
        Route::Text => pages::text(),
        Route::TextPlain => pages::text_plain(),
        Route::Attrs => pages::attrs(),
        Route::Css => pages::css(),
        Route::Table => pages::table(),
        Route::Roundtrip => pages::roundtrip(),
    };

    shell(body)
}

/// The shell every route renders, identical in every respect but the wrapper's contents.
fn shell(body: DomNode) -> DomNode {
    DomElement::new("html")
        .child(DomElement::new("head").child(DomElement::new("title").child_text("ssr-bench")))
        .child(DomElement::new("body").child(DomElement::new("div").attr("id", "rows").child(body)))
        .into()
}
