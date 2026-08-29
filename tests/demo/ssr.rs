//! The two checks in this run that need a real page load.
//!
//! Everything in `tabs` navigates by clicking, which is client-side routing: the server is
//! never asked to render a route, so nothing there touches SSR at all. Both checks below start
//! with a `goto` for that reason, and each puts a fresh console recorder back afterwards
//! because the reload throws away the one that was watching.

use fantoccini::{Client, Locator};

use crate::harness::{find_all, wait_for_no_text, wait_for_text};

/// Load the Counters tab for real, and check what hydration left behind.
///
/// `SsrTest` renders one tree on the server and a deliberately different one in the browser -
/// different depth, different order, different number of children. Hydration has to end up at
/// the browser's tree, and this is the only place in the run that can say whether it did: by
/// the time `tabs::counters` runs the panel a second time, it has been built in the browser
/// from scratch and no server tree was ever in the document to reconcile.
pub async fn hydration(client: &Client, site_url: &str) {
    println!("  -> SSR hydration");

    let url = format!("{site_url}counters");
    client.goto(&url).await.expect("goto /counters failed");
    crate::console::install(client).await;

    // First, because it only appears once the wasm has taken over. Everything after it is a
    // statement about the finished document rather than about one caught mid-hydration - and
    // an assertion about text being *absent* would otherwise pass on a page that had not
    // rendered yet.
    wait_for_text(client, "Rendered by: browser").await;

    // The server's tree is gone: its marker, its extra anchor, and the `<hr/>` with it.
    wait_for_no_text(client, "Rendered by: server").await;
    wait_for_no_text(client, "Only the server draws this link").await;

    // ...and the browser's is what stands, including depth the server never sent.
    wait_for_text(client, "Only the browser draws this, three levels down").await;

    // The two fields carry the same values the server sent, in the order the *browser* asks
    // for. Hydration that paired nodes up by position and stopped there would leave these the
    // way they arrived, which is the other way round.
    let fields = find_all(client, "input").await;
    let values = read_values(&fields).await;
    assert_eq!(
        values,
        vec!["field two".to_string(), "field one".to_string()],
        "hydration should have left the fields in the browser's order, not the server's"
    );

    // The browser renders these two anchors without an href. A server node adopted as-is would
    // still be carrying one.
    for text in ["Shared link one", "Shared link two"] {
        let href = anchor_href(client, text).await;
        assert_eq!(
            href, None,
            "the browser's {text:?} carries no href, so hydration should have removed the \
             server's"
        );
    }
}

async fn read_values(fields: &[fantoccini::elements::Element]) -> Vec<String> {
    let mut values = Vec::new();

    for field in fields {
        values.push(
            field
                .prop("value")
                .await
                .expect("reading a field failed")
                .unwrap_or_default(),
        );
    }

    values
}

/// The `href` of the anchor whose text is `text`, if it still has one.
async fn anchor_href(client: &Client, text: &str) -> Option<String> {
    for anchor in client
        .find_all(Locator::Css("a"))
        .await
        .expect("looking for anchors failed")
    {
        if anchor.text().await.unwrap_or_default().trim() == text {
            return anchor.attr("href").await.expect("reading href failed");
        }
    }

    panic!("no anchor reading {text:?}");
}

/// Load `/fetch` as a real page load and require that the browser did not re-fetch.
///
/// `/fetch` is the right route for this: it fetches during SSR against an absolute URL, so the
/// server's `awc` reaches the stub the same way the browser would. (The Lazy List tab's
/// relative `/api/items` does not survive SSR, which is why the rest of the run avoids `goto`.)
pub async fn fetch_cache(client: &Client, site_url: &str) {
    println!("  -> SSR fetch cache");

    let url = format!("{site_url}fetch");
    client.goto(&url).await.expect("goto /fetch failed");

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
                .is_some_and(|name| name.contains("/fetch/posts"))
        })
        .collect::<Vec<_>>();

    assert!(
        requests.is_empty(),
        "the browser re-fetched what the server had already put in `data-fetch-cache`: {requests:?}\n\
         The posts still rendered, so this is not a visible break - it is the SSR fetch cache \
         no longer being consumed, and every visitor paying a round-trip for it."
    );
}
