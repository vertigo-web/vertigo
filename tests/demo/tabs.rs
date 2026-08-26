//! One function per demo tab: assert its landmarks, work its controls, assert what changed.
//!
//! # Selecting things
//!
//! The demo carries no test attributes on purpose - it doubles as documentation of idiomatic
//! vertigo. Everything here is therefore found by visible text or by structure, and every
//! selector is spelled once so a renamed label is a one-line fix.
//!
//! # Navigating
//!
//! Always by clicking the menu, never with `goto`. `hydrateLink` intercepts clicks on
//! same-origin `<a href>` and pushes history client-side; a `goto` is a full page load, which
//! runs SSR for that route - and SSR fetches with `awc` against the raw URL, so the lazy-list
//! tab's relative `/api/items` fails there. That failure is then *cached*: it rides to the
//! browser in `data-fetch-cache` and `LazyCache` turns it straight into `Resource::Error`.
//!
//! By the same token, links that leave the site must not be clicked: `hydrateLink` ignores
//! `http://` and `https://`, so the YouTube link on Game Of Life and the link inside the SVG
//! would navigate the browser away from the app.

use fantoccini::{Client, Locator};

use crate::harness::{
    body_text, click_by_text, find_all, find_by_text, retype, wait_for_count, wait_for_no_text,
    wait_for_text, wait_until,
};

/// Menu labels, in `Route::label()` order. Also the list the test walks.
pub const TABS: &[&str] = &[
    "Counters",
    "Styling",
    "Sudoku",
    "Input",
    "Github Explorer",
    "Game Of Life",
    "Chat",
    "Todo",
    "Drop File",
    "JS Api Access",
    "List",
    "Lazy List",
    "WS Collection",
    "Svg",
];

/// Click a menu entry and wait for the tab to show something of its own.
pub async fn open(client: &Client, tab: &str, landmark: &str) {
    println!("  -> {tab}");
    click_by_text(client, "a", tab).await;
    wait_for_text(client, landmark).await;
}

/// How many elements match `selector`.
async fn count(client: &Client, selector: &str) -> usize {
    client
        .find_all(Locator::Css(selector))
        .await
        .map(|found| found.len())
        .unwrap_or(0)
}

/// How many elements matching `selector` read exactly `text`.
///
/// Done in one script rather than a round-trip per element: the Sudoku grid alone puts seven
/// hundred candidate divs on the page, and asking about each individually takes minutes.
async fn count_by_text(client: &Client, selector: &str, text: &str) -> usize {
    let script = format!(
        "return Array.from(document.querySelectorAll({selector:?}))\
         .filter((node) => node.textContent.trim() === {text:?}).length;"
    );

    client
        .execute(&script, vec![])
        .await
        .ok()
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as usize
}

/// Every button reading `label` - for the tabs that have more than one of the same name.
async fn buttons_labelled(client: &Client, label: &str) -> Vec<fantoccini::elements::Element> {
    let mut matching = Vec::new();

    for button in find_all(client, "button").await {
        if button.text().await.unwrap_or_default().trim() == label {
            matching.push(button);
        }
    }

    matching
}

/// Click every `<button>` on the page except those whose text is listed.
///
/// This is the "click everything clickable" sweep. It cannot be fully generic: vertigo
/// attaches click handlers with `addEventListener` and leaves no attribute behind, so the many
/// `<div on_click=..>` controls are invisible to any selector. Those are named explicitly by
/// each tab below.
async fn click_all_buttons_except(client: &Client, skip: &[&str]) {
    let buttons = find_all(client, "button").await;

    for button in buttons {
        let label = button.text().await.unwrap_or_default().trim().to_string();

        if skip.iter().any(|skipped| *skipped == label) {
            continue;
        }

        // The DOM is being rebuilt under us as these fire, so a button found a moment ago can
        // be stale by now. That is not a failure - what matters is asserted afterwards.
        let _ = button.click().await;
    }
}

/// Click something that opens a modal dialog, then answer it.
///
/// The session is created with `unhandledPromptBehavior: "ignore"`, so the dialog stands until
/// it is dealt with - and the click itself comes back as an error, because a command cannot
/// complete while a dialog is open. Both are expected.
async fn click_and_accept_dialog(client: &Client, label: &str, answer: Option<&str>) -> String {
    let button = find_by_text(client, "button", label).await;
    let _ = button.click().await;

    let text = client
        .get_alert_text()
        .await
        .unwrap_or_else(|err| panic!("{label:?} should have opened a dialog: {err}"));

    if let Some(answer) = answer {
        client
            .send_alert_text(answer)
            .await
            .expect("answering the prompt failed");
    }

    client
        .accept_alert()
        .await
        .expect("accepting the dialog failed");

    text
}

// -----------------------------------------------------------------------------------------

/// The landing tab. Four counters, their sum, and its double - one `Value` reaching three
/// places, which is the smallest end-to-end check that the reactive graph is alive.
///
/// Also the hydration check: `SsrTest` renders "Content from server" on the server and
/// "Content from browser" in the browser, so seeing the latter proves the wasm took over.
pub async fn counters(client: &Client) {
    open(client, "Counters", "counter1 value").await;

    for (n, value) in [(1, 1), (2, 2), (3, 3), (4, 4)] {
        wait_for_text(client, &format!("counter{n} value = {value}")).await;
    }
    wait_for_text(client, "sum = 10").await;
    wait_for_text(client, "Content from browser").await;

    // One write, three readers: the counter, the sum, and the sum's double - the last through
    // a `Computed` nested inside another, which is its own path through the graph.
    counter_button(client, 2, "up").await;
    wait_for_text(client, "counter2 value = 3").await;
    wait_for_text(client, "sum = 11").await;
    find_by_text(client, "div", "22").await;

    counter_button(client, 2, "down").await;
    wait_for_text(client, "counter2 value = 2").await;
    wait_for_text(client, "sum = 10").await;

    // The div-based controls. None of them changes what is rendered - they write cookies and
    // log - so what is asserted is that the app is still standing afterwards, which the
    // console gate and the closing check between them cover.
    for label in [
        "outer click",
        "Set cookie",
        "Get cookie",
        "Set json cookie",
        "Get json cookie",
        "Get timezone_offset",
        "Get random",
    ] {
        click_by_text(client, "div", label).await;
    }

    // Nested handler with `stop_propagation`.
    click_by_text(client, "button", "Inner click").await;

    // Left until last, because both leave the tab.
    click_by_text(client, "div", "Go to Sudoku").await;
    wait_until("the route to become /sudoku", || async {
        Ok(client.current_url().await?.path() == "/sudoku")
    })
    .await;

    click_by_text(client, "a", "Counters").await;
    wait_for_text(client, "counter1 value").await;

    click_by_text(client, "div", "History back").await;
    wait_until("history_back to return to /sudoku", || async {
        Ok(client.current_url().await?.path() == "/sudoku")
    })
    .await;
}

/// The `up`/`down` button belonging to one counter.
///
/// Anchored on the label rather than counted, so reordering the demo does not silently point
/// this at a different counter. The label div's following sibling is the button row.
async fn counter_button(client: &Client, counter: u32, label: &str) {
    let xpath = format!(
        "//div[starts-with(normalize-space(.), 'counter{counter} value')]\
         /following-sibling::div/button[normalize-space(text())='{label}']"
    );

    client
        .find(Locator::XPath(&xpath))
        .await
        .unwrap_or_else(|err| panic!("counter{counter}'s {label:?} button: {err}"))
        .click()
        .await
        .unwrap_or_else(|err| panic!("clicking counter{counter}'s {label:?}: {err}"));
}

/// Animations, the tooltip, and the tailwind classes.
///
/// The tailwind toggle earns its place beyond smoke value: `<div class="some-external-class"
/// tw={..}>` is a static class and a reactive one on the same element, which is exactly the
/// `Plain -> Merged` promotion the lazy class merger introduced. If that promotion ever drops
/// the value already written, the static class disappears here.
pub async fn styling(client: &Client) {
    open(client, "Styling", "Label with tooltip").await;

    wait_for_text(client, "Spinner:").await;
    wait_for_text(client, "Some tailwind-styled elements").await;
    wait_for_text(client, "Tailwind CSS 4 test").await;
    wait_for_text(client, "Component Taking Tw").await;

    let tw_class = || async {
        client
            .find(Locator::Css("div.some-external-class"))
            .await
            .expect("the tailwind div kept its static class")
            .attr("class")
            .await
            .expect("reading class failed")
            .unwrap_or_default()
    };

    let before = tw_class().await;
    assert!(
        before.contains("some-external-class") && before.contains("bg-green-900"),
        "the tailwind div should carry both its static class and its reactive one, got {before:?}"
    );

    click_by_text(client, "button", "Switch background dd").await;

    wait_until("the tailwind background class to toggle", || async {
        Ok(tw_class().await.contains("bg-green-500"))
    })
    .await;

    let after = tw_class().await;
    assert!(
        after.contains("some-external-class"),
        "the static class must survive a reactive class change, got {after:?}"
    );

    // The progress bar: a `bind_spawn` that ticks a `Value` up and back down, rendering one
    // span per step. Asserted as a round trip rather than on a count at any instant.
    let dots = || async { count(client, "button span span").await };

    // The label sits in a span inside the button - the button itself owns no text - so the
    // click goes to the span and bubbles.
    click_by_text(client, "span", "start the progress bar").await;
    wait_until("the progress bar to advance", || async {
        Ok(dots().await > 5)
    })
    .await;
    wait_until("the progress bar to run back down", || async {
        Ok(dots().await == 0)
    })
    .await;
}

/// An empty 9x9 board, so every cell renders its nine candidates.
///
/// Setting one cell propagates through the solver and removes that candidate from the cell's
/// peers, which is a fan-out no other tab produces: one write, dozens of re-renders. "Clear"
/// must put every one of them back.
pub async fn sudoku(client: &Client) {
    open(client, "Sudoku", "Example 1").await;

    let fives = || async { count_by_text(client, "div", "5").await };

    let before = fives().await;
    assert_eq!(
        before, 81,
        "an empty board should offer the candidate 5 in all 81 cells"
    );

    client
        .find(Locator::XPath("//div[normalize-space(text())='5']"))
        .await
        .expect("a candidate cell reading 5")
        .click()
        .await
        .expect("clicking a candidate failed");

    wait_until(
        "the solver to withdraw 5 from that cell's peers",
        || async { Ok(fives().await < before) },
    )
    .await;

    click_by_text(client, "button", "Clear").await;
    wait_until("Clear to restore every candidate", || async {
        Ok(fives().await == before)
    })
    .await;

    // The remaining example buttons, for the sweep. They log and do nothing else today.
    click_all_buttons_except(client, &["Clear"]).await;
}

/// Text in, text out: one `Value<String>` behind an input, a textarea and a derived length.
pub async fn input(client: &Client) {
    open(client, "Input", "This is input").await;

    wait_for_text(client, "count = 0").await;

    let field = client
        .find(Locator::Css("input"))
        .await
        .expect("the input field");
    retype(&field, "hello").await;
    wait_for_text(client, "count = 5").await;

    // Two buttons writing the same `Value` the input reads.
    click_by_text(client, "button", "set 1").await;
    wait_for_text(client, "count = 5").await;
    wait_until("the input to show the value the button set", || async {
        Ok(field.prop("value").await? == Some("set 1".to_string()))
    })
    .await;

    click_by_text(client, "button", "set 2").await;
    wait_until("the second button to win", || async {
        Ok(field.prop("value").await? == Some("set 2".to_string()))
    })
    .await;

    let textarea = client
        .find(Locator::Css("textarea"))
        .await
        .expect("the textarea");
    retype(&textarea, "abc").await;
    wait_for_text(client, "count = 3").await;
}

/// A fetch, rendered. Points at the local stub rather than api.github.com - see
/// `demo/server/src/stub_api.rs`.
pub async fn github_explorer(client: &Client) {
    open(client, "Github Explorer", "Enter author/repo tuple").await;

    let field = client
        .find(Locator::Css("input"))
        .await
        .expect("the repo field");
    retype(&field, "vertigo-web/vertigo").await;

    click_by_text(client, "button", "Fetch").await;

    // The sha the stub always answers with - the fetch landed and rendered.
    wait_for_text(client, "0000000000000000000000000000000000000001").await;

    // The tab means to echo the repo name back too, with `<text computed={..} />`. It does
    // not, and this is asserted where the value actually lands rather than where the demo
    // meant to put it. Two things go wrong there and neither is this test's to fix:
    //
    //  - `computed` is not special to `dom!`, so it becomes a plain attribute rather than the
    //    element's content;
    //  - `text` is on vertigo's SVG tag list (the workaround for issue #539), so the element
    //    is created in the SVG namespace, and a browser renders nothing for an SVG `<text>`
    //    that is not inside an `<svg>`.
    //
    // Written down here because it is exactly the kind of thing a click-through is supposed to
    // notice: the reactive write is fine, only its destination is wrong.
    wait_until("the repo echo to receive the fetched name", || async {
        Ok(client
            .find(Locator::Css("text"))
            .await?
            .attr("computed")
            .await?
            .as_deref()
            == Some("vertigo-web/vertigo"))
    })
    .await;
}

/// A timer driving a grid of `Value<bool>`, plus the controls around it.
///
/// Start and Stop are asserted as a pair: an unstopped timer keeps re-rendering across every
/// later tab, which would make the rest of the run mean less than it appears to.
pub async fn game_of_life(client: &Client) {
    open(client, "Game Of Life", "Game of life").await;

    // The board counts generations from one, and starts at the delay the state was built with.
    wait_for_text(client, "Year = 1").await;
    wait_for_text(client, "delay = 150").await;

    click_by_text(client, "button", "Random").await;

    click_by_text(client, "button", "Start").await;
    // The label is the timer's own state, so this is the timer confirming it started, and the
    // year moving is it having actually run a generation over 8400 cells.
    wait_for_text(client, "Stop").await;
    wait_for_text(client, "Year = 2").await;

    click_by_text(client, "button", "Stop").await;
    wait_for_text(client, "Start").await;

    let delay = client
        .find(Locator::Css("input"))
        .await
        .expect("the delay field");
    retype(&delay, "500").await;
    click_by_text(client, "button", "Set").await;
    wait_for_text(client, "delay = 500").await;
}

/// The websocket chat, against the demo's own server.
pub async fn chat(client: &Client) {
    open(client, "Chat", "Send").await;

    // Not "turned off": the harness passes `--env ws_chat=..`, so a missing connection here is
    // a real failure rather than a configuration one.
    wait_for_text(client, "Connection active").await;

    let message = "hello from the browser test";

    let field = client
        .find(Locator::Css("input[type=text]"))
        .await
        .expect("the chat input");
    retype(&field, message).await;

    click_by_text(client, "button", "Send").await;

    // Round trip: the server echoes to every connection, including this one.
    wait_for_text(client, message).await;
}

/// Two fetches and three views, against the local stand-in for jsonplaceholder.
pub async fn todo(client: &Client) {
    open(client, "Todo", "post = stub post 1").await;

    wait_for_text(client, "post = stub post 5").await;

    click_by_text(client, "div", "post = stub post 1").await;
    wait_for_text(client, "post_id = 1").await;
    wait_for_text(client, "Comments:").await;
    wait_for_text(client, "comment 1 on post 1").await;

    // The author `<select>`, whose options come from the fetched comments.
    let select = client
        .find(Locator::Css("select"))
        .await
        .expect("the author select");
    select
        .select_by_value("commenter2@example.com")
        .await
        .expect("selecting an author failed");
    wait_for_text(client, "Selected author: commenter2@example.com").await;

    // Clicking an author's name is a third view.
    click_by_text(client, "span", "commenter1@example.com").await;
    wait_for_text(client, "user = commenter1@example.com").await;

    click_by_text(client, "div", "go to post list").await;
    wait_for_text(client, "post = stub post 1").await;
}

/// Landmarks only.
///
/// **Gap, stated rather than hidden:** WebDriver cannot synthesise a file drop, so the one
/// thing this tab exists to demonstrate is not exercised. Everything except the drop itself -
/// the zone rendering, the list that receives files - is checked.
pub async fn drop_file(client: &Client) {
    open(client, "Drop File", "drop file").await;
}

/// Direct JS access, including the three tabs' worth of modal dialogs.
pub async fn js_api_access(client: &Client) {
    open(client, "JS Api Access", "Text to copy:").await;

    wait_for_text(client, "Input with ref:").await;
    wait_for_count(client, "ol li", 200).await;

    // The two "Focus" buttons - one reaching an element by id, one through a `NodeRef` - are
    // done one at a time and checked immediately. They cannot go in the sweep: clicking a
    // button focuses the button, so whatever the sweep pressed last would be the answer.
    let focus_buttons = buttons_labelled(client, "Focus").await;
    assert_eq!(focus_buttons.len(), 2, "both Focus buttons");

    for button in focus_buttons {
        button.click().await.expect("clicking Focus failed");

        wait_until("Focus to move the caret into an input", || async {
            Ok(client
                .execute(
                    "return document.activeElement ? document.activeElement.tagName : '';",
                    vec![],
                )
                .await?
                .as_str()
                == Some("INPUT"))
        })
        .await;
    }

    click_all_buttons_except(
        client,
        // Dialogs are answered separately, below - a sweep would leave them standing and
        // every command after would fail with "unexpected alert open". Focus is done above.
        &["URL", "Referrer", "Ask", "Focus"],
    )
    .await;

    let url = click_and_accept_dialog(client, "URL", None).await;
    assert!(
        url.contains("127.0.0.1"),
        "the URL alert should show the page's own address, got {url:?}"
    );

    click_and_accept_dialog(client, "Referrer", None).await;

    click_and_accept_dialog(client, "Ask", Some("very well")).await;
    wait_for_text(client, "Answer: very well").await;
}

/// One `Value` per row, rendered twice - editable on the left, derived on the right.
///
/// The two panels are built by different machinery (a hand-built `dom_element!` loop and
/// `render_list`), so they diverging is exactly the kind of reconciler bug worth catching.
pub async fn list(client: &Client) {
    open(client, "List", "Left Panel").await;

    wait_for_text(client, "Right Panel").await;
    wait_for_count(client, "input", 3).await;
    assert_eq!(
        count_computed(client).await,
        3,
        "the right panel should mirror the left"
    );

    click_by_text(client, "button", "Add Item").await;
    wait_for_count(client, "input", 4).await;
    wait_until("the right panel to follow the left", || async {
        Ok(count_computed(client).await == 4)
    })
    .await;

    // A write on the left has to reach the derived text on the right.
    let inputs = find_all(client, "input").await;
    retype(&inputs[0], "renamed").await;
    wait_for_text(client, "Computed: renamed").await;

    click_by_text(client, "button", "Remove").await;
    wait_for_count(client, "input", 3).await;
    wait_for_no_text(client, "Computed: renamed").await;
}

/// How many rows the right-hand panel is showing.
async fn count_computed(client: &Client) -> usize {
    body_text(client)
        .await
        .unwrap_or_default()
        .matches("Computed: ")
        .count()
}

/// Optimistic create, update and delete against `/api/items`, proxied to the demo's server.
pub async fn lazy_list(client: &Client) {
    open(client, "Lazy List", "LazyListCache CRUD demo").await;

    for seeded in ["Apples", "Bread", "Coffee"] {
        wait_for_text(client, seeded).await;
    }

    let name_field = client
        .find(Locator::Css("input"))
        .await
        .expect("the new-item field");
    retype(&name_field, "Dates").await;
    click_by_text(client, "button", "Add").await;

    // The row appears optimistically under the placeholder id, then the server's id replaces
    // it. Waiting for the id is what distinguishes "the round trip completed" from "the
    // optimistic row is still sitting there".
    wait_for_text(client, "Dates").await;
    wait_for_text(client, "#4").await;

    click_by_text(client, "button", "Refresh").await;
    wait_for_text(client, "Dates").await;

    // Edit the row just created, so the seeded rows stay put for a re-run.
    edit_row(client, "Dates", "Dates (edited)").await;
    wait_for_text(client, "Dates (edited)").await;

    delete_row(client, "Dates (edited)").await;
    wait_for_no_text(client, "Dates (edited)").await;
}

/// Press "Edit" on the row showing `name`, retype it, and save.
///
/// The row being edited is the only one with a "Save" button, which is what identifies its
/// input - there is another input on this tab, the one that adds new items.
async fn edit_row(client: &Client, name: &str, new_name: &str) {
    row_button(client, name, "Edit").await;

    let field = client
        .find(Locator::XPath(
            "//div[button[normalize-space(text())='Save']]/input",
        ))
        .await
        .expect("the row's edit field");
    retype(&field, new_name).await;

    click_by_text(client, "button", "Save").await;
}

async fn delete_row(client: &Client, name: &str) {
    row_button(client, name, "Delete").await;
}

/// The button in the row whose name cell reads `name`.
async fn row_button(client: &Client, name: &str, label: &str) {
    let xpath = format!(
        "//div[span[normalize-space(text())={name:?}]]\
         /button[normalize-space(text())={label:?}]"
    );

    client
        .find(Locator::XPath(&xpath))
        .await
        .unwrap_or_else(|err| panic!("the {label:?} button on row {name:?}: {err}"))
        .click()
        .await
        .unwrap_or_else(|err| panic!("clicking {label:?} on row {name:?}: {err}"));
}

/// A server-pushed collection: rows arrive over a websocket and the query narrows them.
///
/// Nothing here asserts an exact row count. The server mutates the catalogue on a timer of its
/// own - the stock column ticks and rows come and go - so "the same number as a moment ago" is
/// not a property this tab has. What is asserted is the shape: a query narrows, clearing it
/// widens, and a query that matches nothing says so.
pub async fn ws_collection(client: &Client) {
    open(client, "WS Collection", "Name search:").await;

    let rows = || async { count(client, "tbody tr").await };

    wait_until("the collection to arrive over the websocket", || async {
        Ok(rows().await > 0)
    })
    .await;

    let all = rows().await;

    // Narrowing is done by the server: the client re-subscribes with a new query rather than
    // filtering what it already has.
    let select = client
        .find(Locator::Css("select"))
        .await
        .expect("the kind select");
    select
        .select_by_value("Soprano")
        .await
        .expect("selecting a kind failed");

    wait_until("the kind filter to narrow the collection", || async {
        let narrowed = rows().await;
        Ok(narrowed > 0 && narrowed < all)
    })
    .await;

    let narrowed = rows().await;

    select
        .select_by_value("")
        .await
        .expect("clearing the kind failed");
    wait_until("clearing the kind filter to widen it again", || async {
        Ok(rows().await > narrowed)
    })
    .await;

    let search = client
        .find(Locator::Css("input[type=text]"))
        .await
        .expect("the name search");
    retype(&search, "zzzzz").await;
    wait_for_text(client, "No ukuleles match the current query.").await;

    retype(&search, "").await;
    wait_until("clearing the search to bring the rows back", || async {
        Ok(rows().await > 0)
    })
    .await;
}

/// Namespaced elements. Nothing here is safe to click - the only link leaves the site.
pub async fn svg(client: &Client) {
    open(client, "Svg", "Link in SVG").await;

    assert_eq!(count(client, "svg circle").await, 1, "the svg circle");
    assert_eq!(count(client, "svg path").await, 2, "both svg paths");
}
