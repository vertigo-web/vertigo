//! The checks in this run that need a real request to the server.
//!
//! Everything in `tabs` navigates by clicking, which is client-side routing: the server is
//! never asked for a route, so nothing there touches SSR, the plain-text handler, or the
//! status a route sets. Each check below starts with a `goto` for that reason, and each puts a
//! fresh console recorder back afterwards because the reload throws away the one watching.

use fantoccini::{Client, Locator};

use fantoccini_tests::{Ctx, TestResult};

use crate::harness::{find_all, wait_for_no_text, wait_for_text};

/// Load the Driver tab for real, and check what hydration left behind.
///
/// `SsrTest` renders one tree on the server and a deliberately different one in the browser -
/// different depth, different order, different number of children. Hydration has to end up at
/// the browser's tree, and this is the only place in the run that can say whether it did: by
/// the time `tabs::driver` runs the panel a second time, it has been built in the browser
/// from scratch and no server tree was ever in the document to reconcile.
pub async fn hydration(client: &Client, site_url: &str) -> TestResult {
    println!("  -> SSR hydration");

    let url = format!("{site_url}driver");
    client.goto(&url).await.ctx("goto /driver failed")?;
    crate::console::install(client).await?;

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
    let values = read_values(&fields).await?;
    assert_eq!(
        values,
        vec!["field two".to_string(), "field one".to_string()],
        "hydration should have left the fields in the browser's order, not the server's"
    );

    // The browser renders these two anchors without an href. A server node adopted as-is would
    // still be carrying one.
    for text in ["Shared link one", "Shared link two"] {
        let href = anchor_href(client, text).await?;
        assert_eq!(
            href, None,
            "the browser's {text:?} carries no href, so hydration should have removed the \
             server's"
        );
    }

    Ok(())
}

/// What hydration reported it did, read back from the page.
///
/// `window.__vertigo_hydration` exists for this: the wasm boots asynchronously, so there is no
/// moment at which the test could install a shim on `console.log` and be sure of catching the
/// line hydration prints. A value parked on `window` can be read whenever.
#[derive(Debug)]
struct HydrationReport {
    root_found: bool,
    matched: u64,
    hydratable: u64,
    skipped: u64,
    total: u64,
}

async fn hydration_report(client: &Client) -> TestResult<HydrationReport> {
    let raw = client
        .execute("return window.__vertigo_hydration ?? null;", vec![])
        .await
        .ctx("reading window.__vertigo_hydration failed")?;

    if raw.is_null() {
        return Err("the page published no hydration report - did the wasm boot?".into());
    }

    let field = |name: &str| -> TestResult<u64> {
        raw.get(name)
            .and_then(|value| value.as_u64())
            .ctx(format!("hydration report has no numeric {name:?}: {raw}"))
    };

    Ok(HydrationReport {
        root_found: raw
            .get("rootFound")
            .and_then(|value| value.as_bool())
            .ctx(format!(
                "hydration report has no boolean \"rootFound\": {raw}"
            ))?,
        matched: field("matched")?,
        hydratable: field("hydratable")?,
        skipped: field("skipped")?,
        total: field("total")?,
    })
}

/// Every server-rendered node on a route should be adopted, not rebuilt.
///
/// This is the check that says hydration *happened*. The one above asserts the tree it ends up
/// with, which a full client-side rebuild satisfies just as well - so it passed throughout the
/// period when the first DOM batch reached the browser without `<body>` in it, hydration
/// matched nothing, and the server's markup was thrown away wholesale.
pub async fn hydration_is_complete(client: &Client, site_url: &str) -> TestResult {
    println!("  -> SSR hydration coverage");

    // `/svg` is in here deliberately: SVG elements keep their own casing in `tagName`, so
    // matching them against an uppercased name never succeeded and the whole subtree was
    // deleted and rebuilt.
    for route in ["", "svg"] {
        let url = format!("{site_url}{route}");
        client
            .goto(&url)
            .await
            .ctx(format!("goto /{route} failed"))?;
        crate::console::install(client).await?;

        // Only true once the wasm has taken over, so the report below is the finished one.
        wait_for_text(client, "Game Of Life").await;

        let report = hydration_report(client).await?;

        assert!(
            report.root_found,
            "/{route}: hydration never found <body> in the first DOM batch, so the \
             server-rendered markup was replaced instead of adopted - {report:?}"
        );

        assert!(
            report.hydratable > 0,
            "/{route}: nothing to hydrate, which means this check proves nothing - {report:?}"
        );

        assert_eq!(
            report.matched,
            report.hydratable,
            "/{route}: hydration left {} of {} vnodes unmatched, so that much of the \
             server-rendered page was rebuilt - {report:?}",
            report.hydratable - report.matched,
            report.hydratable,
        );

        println!(
            "     /{route}: {}/{} matched, {} markers skipped, {} vnodes in batch",
            report.matched, report.hydratable, report.skipped, report.total
        );
    }

    Ok(())
}

async fn read_values(fields: &[fantoccini::elements::Element]) -> TestResult<Vec<String>> {
    let mut values = Vec::new();

    for field in fields {
        values.push(
            field
                .prop("value")
                .await
                .ctx("reading a field failed")?
                .unwrap_or_default(),
        );
    }

    Ok(values)
}

/// The `href` of the anchor whose text is `text`, if it still has one.
async fn anchor_href(client: &Client, text: &str) -> TestResult<Option<String>> {
    for anchor in client
        .find_all(Locator::Css("a"))
        .await
        .ctx("looking for anchors failed")?
    {
        if anchor.text().await.unwrap_or_default().trim() == text {
            return anchor.attr("href").await.ctx("reading href failed");
        }
    }

    panic!("no anchor reading {text:?}");
}

/// Load `/fetch` as a real page load and require that the browser did not re-fetch.
///
/// `/fetch` is the right route for this: it fetches during SSR against an absolute URL, so the
/// server's `awc` reaches the stub the same way the browser would. (The Lazy List tab's
/// relative `/api/items` does not survive SSR, which is why the rest of the run avoids `goto`.)
pub async fn fetch_cache(client: &Client, site_url: &str) -> TestResult {
    println!("  -> SSR fetch cache");

    let url = format!("{site_url}fetch");
    client.goto(&url).await.ctx("goto /fetch failed")?;

    // The reload threw away the recorder that was watching the first page load, so put one
    // back before anything below can provoke an error.
    crate::console::install(client).await?;

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
        .ctx("reading resource timings failed")?;

    let timings = timings
        .as_array()
        .ctx("resource timings should be an array")?;

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

    Ok(())
}

/// The plain-text handler: `get_driver().plains(..)` in `demo/app/src/lib.rs`.
///
/// Nothing about it involves the app's DOM - it answers before any route is rendered - so
/// clicking around could never reach it. Read through the browser rather than with an HTTP
/// client so that what is checked is what a crawler would actually be served.
pub async fn robots_txt(client: &Client, site_url: &str) -> TestResult {
    println!("  -> robots.txt");

    let url = format!("{site_url}robots.txt");
    client.goto(&url).await.ctx("goto /robots.txt failed")?;

    let body = crate::harness::body_text(client)
        .await
        .ctx("reading robots.txt failed")?;

    assert!(
        body.contains("User-Agent: *") && body.contains("Disallow: /search"),
        "robots.txt should be the app's plain-text answer, got {body:?}"
    );

    Ok(())
}

/// An unknown address: the app renders Not Found, and the server answers 404.
///
/// The status is the half that has never been covered. `set_status` does nothing unless
/// `is_server()`, so it only takes effect on a real request - and it leaves no trace in the
/// DOM, which means the page rendering correctly says nothing about it. Asked for with a
/// `fetch` from a page already on the origin, because WebDriver will not report the status of
/// a navigation.
pub async fn not_found(client: &Client, site_url: &str) -> TestResult {
    println!("  -> 404");

    let url = format!("{site_url}no-such-page");
    client.goto(&url).await.ctx("goto an unknown page failed")?;
    crate::console::install(client).await?;

    wait_for_text(client, "Page Not Found").await;

    const SCRIPT: &str = r#"
        const [url, done] = arguments;
        fetch(url).then((response) => done(response.status)).catch(() => done(-1));
    "#;

    let status = client
        .execute_async(SCRIPT, vec![serde_json::json!(url)])
        .await
        .ctx("fetching the unknown page failed")?;

    assert_eq!(
        status.as_i64(),
        Some(404),
        "an unknown route should answer 404, not {status}"
    );

    Ok(())
}
