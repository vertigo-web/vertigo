//! Walks the demo app in a real browser: every tab, every control that is safe to press.
//!
//! ```text
//! cargo test --package fantoccini-tests --test demo -- --ignored
//! ```
//!
//! or `just demo-tests`. The harness of `vertigo-testing` starts a chromedriver of its own;
//! `E2E_WEBDRIVER=http://localhost:9515` uses a running one instead, and `E2E_HEADLESS=0`
//! shows the window.
//!
//! This replaces the manual click-through the demo otherwise needs after a change to the
//! reactive graph or the DOM layer. It is not a benchmark: nothing here is timed, and nothing
//! is asserted on how long anything took. What it asserts is that each tab renders what it
//! should, that its controls do what they say, and that the browser console stayed clean.
//!
//! No part of it touches the network. The demo's own API server is started by the test, and
//! the two public APIs the demo otherwise reads - jsonplaceholder and GitHub - are answered by
//! local stand-ins that the app is pointed at with `--env`. See `demo/server/src/stub_api.rs`.
//!
//! A failure saves a screenshot, the page and the console log in
//! `target/e2e/demo/artifacts/`.

mod console;
mod env;
mod helpers;
mod ssr;
mod tabs;

use vertigo_testing::{browser_tests, prelude::*};

type Ctx = vertigo_testing::Ctx<env::DemoEnv>;

async fn demo(ctx: Ctx) -> Result<()> {
    console::allow_in_harness(&ctx);

    ctx.open("/").await?;

    // The menu is rendered by the app, so finding every entry is already a statement that the
    // wasm booted and took over from the server-rendered HTML.
    for tab in tabs::TABS {
        ctx.wait_for_text(tab).await?;
    }

    println!("Walking the tabs");

    // Each tab's console is checked before moving on, so a message names the tab that
    // produced it rather than the run as a whole.
    macro_rules! step {
        ($name:literal, $call:expr) => {{
            $call.await?;
            console::assert_clean(&ctx, $name).await?;
        }};
    }

    step!("Home", tabs::home(&ctx));
    step!("Counters", tabs::counters(&ctx));
    step!("Styling", tabs::styling(&ctx));
    step!("Sudoku", tabs::sudoku(&ctx));
    step!("Input", tabs::input(&ctx));
    step!("Github Explorer", tabs::github_explorer(&ctx));
    step!("Game Of Life", tabs::game_of_life(&ctx));
    step!("Chat", tabs::chat(&ctx));
    step!("Fetch", tabs::fetch(&ctx));
    step!("Drop File", tabs::drop_file(&ctx));
    step!("Driver", tabs::driver(&ctx));
    step!("JS Api Access", tabs::js_api_access(&ctx));
    step!("List", tabs::list(&ctx));
    step!("Lazy List", tabs::lazy_list(&ctx));
    step!("WS Collection", tabs::ws_collection(&ctx));
    step!("Svg", tabs::svg(&ctx));

    // Not a tab: the arrow keys belong to the frame the tabs sit in.
    step!("the arrow keys", tabs::arrow_keys(&ctx));

    // Liveness, independent of the console gate: if the wasm panicked anywhere above, the
    // reactive graph is dead and this write never reaches the DOM.
    println!("Closing check");
    step!("the closing check", tabs::counters(&ctx));

    // Last, because these are the only steps that load pages for real.
    step!("the SSR hydration check", ssr::hydration(&ctx));
    step!(
        "the SSR hydration coverage check",
        ssr::hydration_is_complete(&ctx)
    );
    step!("the SSR fetch cache check", ssr::fetch_cache(&ctx));

    ssr::not_found(&ctx).await?;
    console::assert_clean_except(&ctx, "the 404 check", &[ssr::NOT_FOUND_PATH]).await?;

    // Last of all: it navigates away from the app entirely, so there is no app left to watch
    // afterwards and nothing below it could rely on one.
    step!("the robots.txt check", ssr::robots_txt(&ctx));

    Ok(())
}

fn main() {
    // Run on request only (`--ignored`), like the other browser suites: it needs a browser.
    let tests = browser_tests![demo]
        .into_iter()
        .map(|test| TestCase {
            ignored: true,
            ..test
        })
        .collect();

    run_suite("demo", tests);
}
