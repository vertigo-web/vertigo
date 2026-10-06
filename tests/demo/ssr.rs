//! The checks in this run that need a real request to the server.
//!
//! Everything in `tabs` navigates by clicking, which is client-side routing: the server is
//! never asked for a route, so nothing there touches SSR, the plain-text handler, or the
//! status a route sets. Each check below starts with a real page load for that reason.

use vertigo_testing::{prelude::*, thirtyfour::WebElement};

use crate::{
    Ctx,
    helpers::{check_eq, find_all},
};

/// Load the Driver tab for real, and check what hydration left behind.
///
/// `SsrTest` renders one tree on the server and a deliberately different one in the browser -
/// different depth, different order, different number of children. Hydration has to end up at
/// the browser's tree, and this is the only place in the run that can say whether it did: by
/// the time `tabs::driver` runs the panel a second time, it has been built in the browser
/// from scratch and no server tree was ever in the document to reconcile.
pub async fn hydration(ctx: &Ctx) -> Result<()> {
    println!("  -> SSR hydration");

    ctx.driver
        .goto(ctx.url("/driver"))
        .await
        .context("goto /driver failed")?;

    // First, because it only appears once the wasm has taken over. Everything after it is a
    // statement about the finished document rather than about one caught mid-hydration - and
    // an assertion about text being *absent* would otherwise pass on a page that had not
    // rendered yet.
    ctx.wait_for_text("Rendered by: browser").await?;

    // The server's tree is gone: its marker, its extra anchor, and the `<hr/>` with it.
    ctx.wait_for_no_text("Rendered by: server").await?;
    ctx.wait_for_no_text("Only the server draws this link")
        .await?;

    // ...and the browser's is what stands, including depth the server never sent.
    ctx.wait_for_text("Only the browser draws this, three levels down")
        .await?;

    // The two fields carry the same values the server sent, in the order the *browser* asks
    // for. Hydration that paired nodes up by position and stopped there would leave these the
    // way they arrived, which is the other way round.
    let fields = find_all(ctx, "input").await?;
    check_eq(
        read_values(&fields).await?,
        ["field two", "field one"],
        "hydration should have left the fields in the browser's order, not the server's",
    )?;

    // The browser renders these two anchors without an href. A server node adopted as-is would
    // still be carrying one.
    for text in ["Shared link one", "Shared link two"] {
        check_eq(
            anchor_href(ctx, text).await?,
            None,
            format!(
                "the browser's {text:?} carries no href, so hydration should have removed the \
                 server's"
            ),
        )?;
    }

    Ok(())
}

/// Every server-rendered node on a route should be adopted, not rebuilt.
///
/// This is the check that says hydration *happened*. The one above asserts the tree it ends up
/// with, which a full client-side rebuild satisfies just as well - so it passed throughout the
/// period when the first DOM batch reached the browser without `<body>` in it, hydration
/// matched nothing, and the server's markup was thrown away wholesale.
pub async fn hydration_is_complete(ctx: &Ctx) -> Result<()> {
    println!("  -> SSR hydration coverage");

    // `/svg` is in here deliberately: SVG elements keep their own casing in `tagName`, so
    // matching them against an uppercased name never succeeded and the whole subtree was
    // deleted and rebuilt.
    for route in ["/", "/svg"] {
        // Waits for the report itself. Waiting for a text instead, "Game Of Life" say, races
        // the boot: the menu is in the server-rendered HTML already, so the text is there
        // before the wasm is, and the report sometimes is not yet.
        ctx.open(route)
            .await
            .with_context(|| format!("loading {route} failed"))?;

        let report = ctx.hydration_report().await?;

        ensure!(
            report.root_found,
            "{route}: hydration never found <body> in the first DOM batch, so the \
             server-rendered markup was replaced instead of adopted - {report:?}"
        );

        ensure!(
            report.hydratable > 0,
            "{route}: nothing to hydrate, which means this check proves nothing - {report:?}"
        );

        check_eq(
            report.matched,
            report.hydratable,
            format!(
                "{route}: hydration left {} of {} vnodes unmatched, so that much of the \
                 server-rendered page was rebuilt - {report:?}",
                report.hydratable.saturating_sub(report.matched),
                report.hydratable,
            ),
        )?;

        println!(
            "     {route}: {}/{} matched, {} markers skipped, {} vnodes in batch",
            report.matched, report.hydratable, report.skipped, report.total
        );
    }

    Ok(())
}

async fn read_values(fields: &[WebElement]) -> Result<Vec<String>> {
    let mut values = Vec::new();

    for field in fields {
        values.push(
            field
                .prop("value")
                .await
                .context("reading a field failed")?
                .unwrap_or_default(),
        );
    }

    Ok(values)
}

/// The `href` of the anchor whose text is `text`, if it still has one.
async fn anchor_href(ctx: &Ctx, text: &str) -> Result<Option<String>> {
    for anchor in ctx
        .find_all(By::Css("a"))
        .await
        .context("looking for anchors failed")?
    {
        if anchor.text().await.unwrap_or_default().trim() == text {
            return anchor.attr("href").await.context("reading href failed");
        }
    }

    bail!("no anchor reading {text:?}")
}

/// Load `/fetch` as a real page load and require that the browser did not re-fetch.
///
/// `/fetch` is the right route for this: it fetches during SSR against an absolute URL, so the
/// server's `awc` reaches the stub the same way the browser would.
pub async fn fetch_cache(ctx: &Ctx) -> Result<()> {
    println!("  -> SSR fetch cache");

    // Until the wasm has started, nothing in the browser could have fetched anything, and the
    // check below would pass for nothing. The posts alone don't tell: they are in the
    // server-rendered HTML.
    ctx.open("/fetch").await.context("loading /fetch failed")?;

    // Rendered at all - so the server prefetched, embedded, and the browser decoded. A cache
    // that arrived as `Resource::Error` would leave the list empty and fail here instead.
    ctx.wait_for_text("post = stub post 1").await?;
    ctx.wait_for_text("post = stub post 5").await?;

    // ...and rendered without asking for them again. The request carries `ttl_minutes(10)`, so
    // a hit cannot expire mid-run: any request here means the cache was missed, not refreshed.
    let timings = ctx
        .js("return performance.getEntriesByType('resource').map((entry) => entry.name);")
        .await
        .context("reading resource timings failed")?;

    let timings = timings
        .as_array()
        .context("resource timings should be an array")?;

    // Resource timing has to be recording something, or the filter below is vacuous and this
    // check would pass however broken the cache was. The page loads a `.wasm` at minimum.
    ensure!(
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

    ensure!(
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
pub async fn robots_txt(ctx: &Ctx) -> Result<()> {
    println!("  -> robots.txt");

    ctx.driver
        .goto(ctx.url("/robots.txt"))
        .await
        .context("goto /robots.txt failed")?;

    let body = ctx.page_text().await.context("reading robots.txt failed")?;

    ensure!(
        body.contains("User-Agent: *") && body.contains("Disallow: /search"),
        "robots.txt should be the app's plain-text answer, got {body:?}"
    );

    Ok(())
}

/// The address [`not_found`] asks for. Its 404s in the console are that check's own doing.
pub const NOT_FOUND_PATH: &str = "/no-such-page";

/// An unknown address: the app renders Not Found, and the server answers 404.
///
/// The status is the half that has never been covered. `set_status` does nothing unless
/// `is_server()`, so it only takes effect on a real request - and it leaves no trace in the
/// DOM, which means the page rendering correctly says nothing about it. Asked for with a
/// `fetch` from a page already on the origin, because WebDriver will not report the status of
/// a navigation.
pub async fn not_found(ctx: &Ctx) -> Result<()> {
    println!("  -> 404");

    let url = ctx.url(NOT_FOUND_PATH);
    ctx.driver
        .goto(&url)
        .await
        .context("goto an unknown page failed")?;

    ctx.wait_for_text("Page Not Found").await?;

    const SCRIPT: &str = r#"
        const [url, done] = arguments;
        fetch(url).then((response) => done(response.status)).catch(() => done(-1));
    "#;

    let status = ctx
        .driver
        .execute_async(SCRIPT, vec![json!(url)])
        .await
        .context("fetching the unknown page failed")?
        .json()
        .clone();

    check_eq(
        status.as_i64(),
        Some(404),
        format!("an unknown route should answer 404, not {status}"),
    )
}
