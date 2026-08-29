//! Walks the demo app in a real browser: every tab, every control that is safe to press.
//!
//! Requires a WebDriver on localhost:9515. Run with:
//!
//! ```text
//! cargo test --package fantoccini-tests --test demo -- --ignored --nocapture
//! ```
//!
//! or `task demo-tests`.
//!
//! This replaces the manual click-through the demo otherwise needs after a change to the
//! reactive graph or the DOM layer. It is not a benchmark: nothing here is timed, and nothing
//! is asserted on how long anything took. What it asserts is that each tab renders what it
//! should, that its controls do what they say, and that the browser console stayed clean.
//!
//! No part of it touches the network. The demo's own API server is started by the test, and
//! the two public APIs the demo otherwise reads - jsonplaceholder and GitHub - are answered by
//! local stand-ins that the app is pointed at with `--env`. See `demo/server/src/stub_api.rs`.

mod console;
mod harness;
mod ssr;
mod tabs;

use harness::{Harness, wait_for_text};

#[tokio::test]
#[ignore]
async fn demo() {
    let harness = Harness::start().await;
    let client = &harness.client;

    console::install(client).await;

    // The menu is rendered by the app, so finding every entry is already a statement that the
    // wasm booted and took over from the server-rendered HTML.
    for tab in tabs::TABS {
        wait_for_text(client, tab).await;
    }

    println!("Walking the tabs");

    // Each tab's console tally is checked before moving on, so a message names the tab that
    // produced it rather than the run as a whole.
    macro_rules! step {
        ($name:literal, $call:expr) => {{
            $call.await;
            console::assert_clean(client, $name).await;
        }};
    }

    step!("Home", tabs::home(client));
    step!("Counters", tabs::counters(client));
    step!("Styling", tabs::styling(client));
    step!("Sudoku", tabs::sudoku(client));
    step!("Input", tabs::input(client));
    step!("Github Explorer", tabs::github_explorer(client));
    step!("Game Of Life", tabs::game_of_life(client));
    step!("Chat", tabs::chat(client));
    step!("Fetch", tabs::fetch(client));
    step!("Drop File", tabs::drop_file(client));
    step!("Driver", tabs::driver(client));
    step!("JS Api Access", tabs::js_api_access(client));
    step!("List", tabs::list(client));
    step!("Lazy List", tabs::lazy_list(client));
    step!("WS Collection", tabs::ws_collection(client));
    step!("Svg", tabs::svg(client));

    // Not a tab: the arrow keys belong to the frame the tabs sit in.
    step!("the arrow keys", tabs::arrow_keys(client));

    // Liveness, independent of the console gate: if the wasm panicked anywhere above, the
    // reactive graph is dead and this write never reaches the DOM.
    println!("Closing check");
    step!("the closing check", tabs::counters(client));

    // Last, because these are the only steps that reload the page - which is the whole point
    // of them, and which also discards the recorder installed above. Each puts a fresh one
    // back, so the console gate still applies to the pages they load.
    step!(
        "the SSR hydration check",
        ssr::hydration(client, &harness.site_url)
    );
    step!(
        "the SSR fetch cache check",
        ssr::fetch_cache(client, &harness.site_url)
    );
    step!("the 404 check", ssr::not_found(client, &harness.site_url));

    // Last of all: it navigates away from the app entirely, so there is no app left to watch
    // afterwards and nothing below it could rely on one.
    step!(
        "the robots.txt check",
        ssr::robots_txt(client, &harness.site_url)
    );

    harness.finish().await;
}
