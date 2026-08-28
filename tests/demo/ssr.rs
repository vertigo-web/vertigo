//! The one check in this run that the responses the server fetched reach the browser.
//!
//! Everything in `tabs` navigates by clicking, which is client-side routing and never produces
//! a server-rendered page. So none of it touches `data-fetch-cache`: the attribute the server
//! hangs its prefetched responses on, which `LazyCache::new` reads instead of fetching again.
//!
//! That path is easy to break silently. If the cache stops being read, every tab still
//! renders, because the browser simply fetches what it had already been handed; the only
//! symptom is an extra round-trip, and nothing here times anything. So the assertion below is
//! about the *absence* of a request rather than about what rendered.

use fantoccini::Client;

use crate::harness::wait_for_text;

/// Load `/todo` as a real page load and require that the browser did not re-fetch.
///
/// `/todo` is the right route for this: it fetches during SSR against an absolute URL, so the
/// server's `awc` reaches the stub the same way the browser would. (The Lazy List tab's
/// relative `/api/items` does not survive SSR, which is why the rest of the run avoids `goto`.)
pub async fn fetch_cache(client: &Client, site_url: &str) {
    println!("  -> SSR fetch cache");

    let url = format!("{site_url}todo");
    client.goto(&url).await.expect("goto /todo failed");

    // The reload threw away the recorder that was watching the first page load, so put one
    // back before anything below can provoke an error.
    crate::console::install(client).await;

    // Rendered at all - so the server prefetched, embedded, and the browser decoded. A cache
    // that arrived as `Resource::Error` would leave the list empty and fail here instead.
    wait_for_text(client, "post = stub post 1").await;
    wait_for_text(client, "post = stub post 5").await;

    // ...and rendered without asking for them again. The request carries `ttl_minutes(10)`, so
    // a hit cannot expire mid-run: any request here means the cache was missed, not refreshed.
    let timings = client
        .execute(
            "return performance.getEntriesByType('resource').map((entry) => entry.name);",
            vec![],
        )
        .await
        .expect("reading resource timings failed");

    let timings = timings
        .as_array()
        .expect("resource timings should be an array");

    // Resource timing has to be recording something, or the filter below is vacuous and this
    // check would pass however broken the cache was. The page loads a `.wasm` at minimum.
    assert!(
        timings
            .iter()
            .any(|name| name.as_str().is_some_and(|name| name.ends_with(".wasm"))),
        "resource timing recorded no wasm request, so it is not recording fetches either \
         and this check proves nothing. Recorded: {timings:?}"
    );

    let requests = timings
        .iter()
        .filter(|name| {
            name.as_str()
                .is_some_and(|name| name.contains("/todo/posts"))
        })
        .collect::<Vec<_>>();

    assert!(
        requests.is_empty(),
        "the browser re-fetched what the server had already put in `data-fetch-cache`: {requests:?}\n\
         The posts still rendered, so this is not a visible break - it is the SSR fetch cache \
         no longer being consumed, and every visitor paying a round-trip for it."
    );
}
