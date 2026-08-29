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
use serde_json::json;

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

    let (label_left, label_width, popup_left) = tooltip_geometry(client).await;
    assert!(
        popup_left >= label_left + label_width,
        "the tooltip popup should sit past the right edge of its label; \
         label starts at {label_left} and is {label_width} wide, popup starts at {popup_left}"
    );

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

/// Where the tooltip's label and popup are: `(label left, label width, popup left)`.
///
/// The popup is `visibility: hidden` until hovered, which still reserves its box - so its
/// position is readable without having to drive a hover.
async fn tooltip_geometry(client: &Client) -> (f64, f64, f64) {
    const SCRIPT: &str = r#"
        const popup = Array.from(document.querySelectorAll('span'))
            .find((node) => node.textContent.trim() === 'This is content of the tooltip');
        if (!popup) { return null; }
        const label = popup.parentElement.getBoundingClientRect();
        return [label.left, label.width, popup.getBoundingClientRect().left];
    "#;

    let values = client
        .execute(SCRIPT, vec![])
        .await
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_f64())
        .collect::<Vec<_>>();

    match values.as_slice() {
        [label_left, label_width, popup_left] => (*label_left, *label_width, *popup_left),
        _ => panic!("could not measure the tooltip; got {values:?}"),
    }
}

/// An empty 9x9 board, with and without the solver's hints.
///
/// The two modes are different enough to be worth walking separately. With hints on, setting a
/// cell propagates through the solver and withdraws that candidate from the cell's peers - a
/// fan-out no other tab produces, one write and dozens of re-renders. With them off the cells
/// offer every digit and say nothing, which is the only way to enter something illegal.
pub async fn sudoku(client: &Client) {
    open(client, "Sudoku", "Easy").await;

    let fives = || async { count_by_text(client, "div", "5").await };

    // Hints start off.
    wait_for_text(client, "Hints: off").await;
    wait_for_text(client, "Empty board").await;
    assert_eq!(filled_cells(client).await, 0, "the board starts empty");
    assert_eq!(
        fives().await,
        81,
        "with the hints off every cell should offer a 5, possible or not"
    );

    // Nothing the solver knows is on show, so a digit it would rule out can still be entered.
    // That is the only route to a conflict: with hints on, the only digits a cell offers are
    // the ones that are still legal in it.
    let board_at = board_left(client).await;

    click_digit(client, "5", 0).await;
    wait_for_text(client, "1 of 81 filled").await;

    // The next cell still offering a 5 is the one beside it, in the same 3x3 block.
    click_digit(client, "5", 0).await;
    wait_for_text(client, "Conflict: a digit repeats").await;

    // That message is much wider than the counts it replaces, and it sits in the panel beside
    // the board. The panel is a fixed width so the board cannot be pushed sideways by it.
    assert_eq!(
        board_left(client).await,
        board_at,
        "the board should stay put while the status message is a wide one"
    );

    click_by_text(client, "button", "Clear").await;
    wait_for_text(client, "Empty board").await;
    assert_eq!(
        board_left(client).await,
        board_at,
        "...and be back where it started once the message goes"
    );

    // Hints on, and now the solver drives what an empty cell shows.
    click_by_text(client, "div", "Hints: off").await;
    wait_for_text(client, "Hints: on").await;

    let before = fives().await;
    assert_eq!(
        before, 81,
        "an empty board should offer the candidate 5 in all 81 cells"
    );

    click_digit(client, "5", 0).await;

    wait_until(
        "the solver to withdraw 5 from that cell's peers",
        || async { Ok(fives().await < before) },
    )
    .await;

    // The status line is a second `Computed` over the same 81 cells, so it agreeing with what
    // was rendered is two independent readings of one write.
    wait_for_text(client, "1 of 81 filled").await;

    // A value the player entered can be taken back off, and carries the "X" that does it.
    assert_eq!(
        count_by_text(client, "div", "X").await,
        1,
        "an entered value should offer its delete control"
    );

    click_by_text(client, "button", "Clear").await;
    wait_until("Clear to restore every candidate", || async {
        Ok(fives().await == before)
    })
    .await;
    wait_for_text(client, "Empty board").await;

    // The three example boards. Each used to be a `log::info!` and nothing else.
    //
    // Every one of them writes all 81 cells, blanks included, so the count after a load is
    // that board's own given count and never anything left over from the board before it -
    // which is why these run back to back with no Clear in between.
    for (label, givens) in [("Easy", 36), ("Medium", 30), ("Hard", 23)] {
        click_by_text(client, "button", label).await;

        wait_until(&format!("{label} to put {givens} givens on the board"), {
            || async { Ok(filled_cells(client).await == givens) }
        })
        .await;

        // ...and that the solver saw them. A board with givens on it cannot still be
        // offering every candidate in every cell.
        assert!(
            fives().await < before,
            "{label} should have withdrawn candidates from the givens' peers"
        );

        wait_for_text(client, &format!("{givens} of 81 filled")).await;

        // Givens belong to the puzzle, so none of them offers the delete control that an
        // entered value does.
        assert_eq!(
            count_by_text(client, "div", "X").await,
            0,
            "{label}'s givens should not be deletable"
        );
    }

    // Hard is on the board now, so walk back to the first one and read it off the screen.
    //
    // The counts above say the right number of cells were filled; they say nothing about
    // *where*. A board loaded along the wrong axis is still 36 givens, still consistent, and
    // still uniquely solvable - a sudoku transposes to a sudoku - so it would pass everything
    // above while rendering the puzzle mirrored along its diagonal.
    click_by_text(client, "button", "Easy").await;
    wait_until("Easy to come back", || async {
        Ok(filled_cells(client).await == 36)
    })
    .await;

    assert_eq!(
        read_board(client).await,
        [
            "...26.7.1",
            "68..7..9.",
            "19...45..",
            "82.1...4.",
            "..46.29..",
            ".5...3.28",
            "..93...74",
            ".4..5..36",
            "7.3.18...",
        ],
        "Easy should render the board the way `examples::EASY` is written"
    );

    click_by_text(client, "button", "Clear").await;
    wait_until("Clear to take the last board off again", || async {
        Ok(filled_cells(client).await == 0 && fives().await == before)
    })
    .await;
}

/// The x position of the Sudoku board.
///
/// Found by shape rather than by any marker on it: the board is the one element holding nine
/// children that each hold nine of their own. A group div has nine children too, but each of
/// those holds one, so only the board matches.
async fn board_left(client: &Client) -> i64 {
    const SCRIPT: &str = r#"
        const board = Array.from(document.querySelectorAll('div')).find((node) => {
            const groups = Array.from(node.children);
            return groups.length === 9
                && groups.every((group) => group.children.length === 9);
        });
        return board === undefined ? null : Math.round(board.getBoundingClientRect().left);
    "#;

    client
        .execute(SCRIPT, vec![])
        .await
        .ok()
        .and_then(|value| value.as_i64())
        .unwrap_or_else(|| panic!("could not find the Sudoku board"))
}

/// Click `digit` in the nth empty Sudoku cell that is offering it.
///
/// A cell offering a digit sets it at the inherited size, whether that is one of the solver's
/// candidates or one of the plain choices shown when the hints are off. A cell that already
/// *holds* that digit sets it at 30px - so the size is what keeps this pointed at somewhere
/// still to fill rather than at what was just entered.
async fn click_digit(client: &Client, digit: &str, nth: usize) {
    const SCRIPT: &str = r#"
        const [digit, nth] = arguments;
        const own = (node) => Array.from(node.childNodes)
            .filter((child) => child.nodeType === Node.TEXT_NODE)
            .map((child) => child.textContent)
            .join('')
            .trim();

        const offered = Array.from(document.querySelectorAll('div')).filter((node) =>
            own(node) === digit && getComputedStyle(node).fontSize !== '30px');

        if (offered.length <= nth) { return offered.length; }
        offered[nth].click();
        return null;
    "#;

    let available = client
        .execute(SCRIPT, vec![json!(digit), json!(nth)])
        .await
        .expect("looking for a cell offering that digit failed");

    assert!(
        available.is_null(),
        "wanted the cell at index {nth} offering {digit:?}, but only {available} cells offer it"
    );
}

/// How many cells on the Sudoku board hold a value.
///
/// Own text will not separate a filled cell from a candidate cell - both own just their digit -
/// and neither will the delete "X", which only entered values carry now that givens are
/// protected. What does separate them is how they are drawn: a filled cell is set in 30px on
/// the cell's own background, while the candidates that share that size (the one-remaining and
/// last-possible hints) sit on green.
async fn filled_cells(client: &Client) -> usize {
    const SCRIPT: &str = r#"
        return Array.from(document.querySelectorAll('div')).filter((node) => {
            const own = Array.from(node.childNodes)
                .filter((child) => child.nodeType === Node.TEXT_NODE)
                .map((child) => child.textContent)
                .join('')
                .trim();
            if (!/^[1-9]$/.test(own)) { return false; }
            const style = getComputedStyle(node);
            return style.fontSize === '30px' && style.backgroundColor === 'rgba(0, 0, 0, 0)';
        }).length;
    "#;

    client
        .execute(SCRIPT, vec![])
        .await
        .ok()
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| panic!("could not count the filled Sudoku cells")) as usize
}

/// The Sudoku board as it is laid out on screen: nine rows of nine, `.` for a blank.
///
/// Read from geometry rather than from the DOM tree, because the tree does not have rows in
/// it - the board is nine 3x3 blocks, each its own grid - so reconstructing a row by walking
/// parents would just re-derive the same index arithmetic the app used, and agree with it
/// whether or not that arithmetic is right.
///
/// Filled cells are picked out the same way [`filled_cells`] picks them out.
///
/// Rows and columns come from ranking the distinct top and left offsets, so uneven block
/// borders cannot shift a cell into the wrong slot. It does assume every row and column of the
/// board being read holds at least one given - true of `examples::EASY`, which is the only
/// board this is used on.
async fn read_board(client: &Client) -> Vec<String> {
    const SCRIPT: &str = r#"
        const ownText = (node) => Array.from(node.childNodes)
            .filter((child) => child.nodeType === Node.TEXT_NODE)
            .map((child) => child.textContent)
            .join('')
            .trim();

        const cells = Array.from(document.querySelectorAll('div'))
            .filter((node) => {
                if (!/^[1-9]$/.test(ownText(node))) { return false; }
                const style = getComputedStyle(node);
                return style.fontSize === '30px'
                    && style.backgroundColor === 'rgba(0, 0, 0, 0)';
            })
            .map((node) => {
                const box = node.getBoundingClientRect();
                return {
                    top: Math.round(box.top),
                    left: Math.round(box.left),
                    value: ownText(node),
                };
            });

        const ranks = (values) => [...new Set(values)].sort((a, b) => a - b);
        const tops = ranks(cells.map((cell) => cell.top));
        const lefts = ranks(cells.map((cell) => cell.left));

        const board = Array.from({ length: 9 }, () => Array(9).fill('.'));
        for (const cell of cells) {
            board[tops.indexOf(cell.top)][lefts.indexOf(cell.left)] = cell.value;
        }
        return board.map((row) => row.join(''));
    "#;

    client
        .execute(SCRIPT, vec![])
        .await
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect()
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

    // Characters, not bytes. The count used to be `String::len()`, which reads seven here.
    retype(&textarea, "żółw").await;
    wait_for_text(client, "count = 4").await;
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

    // The tab echoes back which repo it is showing. This used to be `<text computed={..} />`,
    // which rendered nothing at all, and the assertion here had to read the value off an
    // attribute instead: `computed` is not special to `dom!` so it became a plain attribute
    // rather than the element's content, and `text` is on vertigo's SVG tag list (the
    // workaround for issue #539) so the element was created in the SVG namespace, where a
    // browser paints nothing unless it sits inside an `<svg>`.
    wait_for_text(client, "Showing: vertigo-web/vertigo").await;
}

/// A timer driving a grid of `Value<bool>`, plus the controls around it.
///
/// Start and Stop are asserted as a pair: an unstopped timer keeps re-rendering across every
/// later tab, which would make the rest of the run mean less than it appears to.
pub async fn game_of_life(client: &Client) {
    open(client, "Game Of Life", "Game of life").await;

    // The board counts generations from one, and starts at the delay the state was built with.
    wait_for_text(client, "Year = 1").await;
    wait_for_delay(client, 50).await;

    // Population is a `Computed` over all 8400 cells - the widest fan-in on this tab, and the
    // handle everything below uses to say what the board is doing.
    click_by_text(client, "button", "Clear").await;
    assert_eq!(
        life_population(client).await,
        0,
        "Clear should empty the board"
    );

    // A spaceship: the same five cells for as long as it runs, wherever they have got to.
    // A load that dropped or duplicated a cell, or a generation that mis-stepped, would not
    // hold this.
    load_pattern(client, "Glider", 5).await;

    for year in 2..=5 {
        click_by_text(client, "button", "Step").await;
        wait_until(&format!("Step to reach year {year}"), || async {
            Ok(life_year(client).await == year)
        })
        .await;

        assert_eq!(
            life_population(client).await,
            5,
            "a glider keeps its five cells through year {year}"
        );
    }

    // A lightweight spaceship breathes between 9 and 12 cells as it travels, so the population
    // label changes width every generation. The counters reserve room for their widest value
    // precisely so that does not re-flow the header: without it, Start, Step and the delay
    // field slide left and right on every frame while the board runs.
    load_pattern(client, "Spaceship", 9).await;
    let anchored = element_left(client, "button", "Step").await;

    for expected in [12, 9, 12] {
        click_by_text(client, "button", "Step").await;
        wait_until(
            &format!("the spaceship to reach {expected} cells"),
            || async { Ok(life_population(client).await == expected) },
        )
        .await;

        assert_eq!(
            element_left(client, "button", "Step").await,
            anchored,
            "the controls should stay put as the population label changes width"
        );
    }

    // An oscillator with a period of three: 48 -> 56 -> 72 -> 48. Asserted through a whole
    // cycle, so it is the shape coming back rather than a count that happens to match.
    load_pattern(client, "Pulsar", 48).await;

    for expected in [56, 72, 48] {
        click_by_text(client, "button", "Step").await;
        wait_until(
            &format!("the pulsar to reach a population of {expected}"),
            || async { Ok(life_population(client).await == expected) },
        )
        .await;
    }

    // Seven cells that vanish completely, and only at generation 130. Stepped rather than
    // timed, so this is the rule being applied 129 times and not a race with a timer.
    load_pattern(client, "Diehard", 7).await;

    // The board counts the pattern as it was loaded as year 1, so its year N is generation
    // N - 1. Both sides of the transition are asserted, which is what makes this "at 130" and
    // not merely "eventually".
    step_times(client, 129).await;
    assert_eq!(
        (life_year(client).await, life_population(client).await),
        (130, 2),
        "diehard should still have two cells one generation short of the end"
    );

    step_times(client, 1).await;
    assert_eq!(
        (life_year(client).await, life_population(client).await),
        (131, 0),
        "diehard should be gone at generation 130 - the board's year 131"
    );

    // A board of still lifes and oscillators, none of them close enough to disturb another.
    // Every period in it divides 30, so after thirty generations the whole thing is back
    // exactly where it started - which a single stray interaction would destroy.
    load_pattern(client, "Menagerie", 103).await;

    step_times(client, 30).await;

    assert_eq!(
        (life_year(client).await, life_population(client).await),
        (31, 103),
        "the menagerie should come back to its own population after 30 generations"
    );

    // A gun grows without bound, which no still life or oscillator does.
    load_pattern(client, "Glider gun", 36).await;

    click_by_text(client, "button", "Start").await;
    wait_for_text(client, "Stop").await;
    wait_until(
        "the glider gun to grow past its initial 36 cells",
        || async { Ok(life_population(client).await > 60) },
    )
    .await;

    click_by_text(client, "button", "Stop").await;
    wait_for_text(client, "Start").await;

    // Random fills roughly 35% of 8400, so about 2940 cells. It used to fill by
    // `(y * 2 + (x + 4)) % 2 == 0`, which reduces to `x % 2 == 0` - every other column, filled
    // top to bottom. That is exactly 4200 cells, and a still life besides.
    click_by_text(client, "button", "Random").await;
    let live = life_population(client).await;
    assert!(
        (2500..3400).contains(&live),
        "Random should fill about 35% of the 8400 cells, got {live}"
    );

    click_by_text(client, "button", "Start").await;
    wait_for_text(client, "Stop").await;
    wait_for_text(client, "Year = 2").await;

    // ...and the generation it ran changed something. A still life would leave this equal.
    wait_until("the population to move between generations", || async {
        Ok(life_population(client).await != live)
    })
    .await;

    click_by_text(client, "button", "Stop").await;
    wait_for_text(client, "Start").await;

    let delay = client
        .find(Locator::Css("input"))
        .await
        .expect("the delay field");

    // A non-numeric delay used to parse as `unwrap_or_default()` - zero - on every keystroke,
    // and the Set button then handed that zero to `set_interval` without anyone asking for it.
    // Refused now, and said so.
    retype(&delay, "abc").await;
    click_by_text(client, "button", "Set").await;
    wait_for_text(client, "Delay not set").await;
    wait_for_delay(client, 50).await;

    // Zero is a legitimate answer, not a typo: being a number is the whole of the check, and
    // the very short delays are the ones worth watching.
    retype(&delay, "0").await;
    click_by_text(client, "button", "Set").await;
    wait_for_delay(client, 0).await;
    wait_for_no_text(client, "Delay not set").await;

    // ...and the board is still drivable at that end of the range. This is the assertion that
    // matters for the small delays: a generation over 8400 cells takes longer than the
    // interval, so the timer is re-entered as fast as the browser will schedule it, and the
    // question is whether the Stop button still gets a turn.
    retype(&delay, "5").await;
    click_by_text(client, "button", "Set").await;
    wait_for_delay(client, 5).await;

    let before = life_year(client).await;
    click_by_text(client, "button", "Start").await;
    wait_until("the board to run several generations at a 5 ms delay", {
        || async { Ok(life_year(client).await > before + 2) }
    })
    .await;

    click_by_text(client, "button", "Stop").await;
    wait_for_text(client, "Start").await;

    // Left somewhere unremarkable: an unstopped fast timer would make every later tab slower
    // and noisier than it should be, and the run has eight more to go.
    retype(&delay, "500").await;
    click_by_text(client, "button", "Set").await;
    wait_for_delay(client, 500).await;
}

/// Wait for the board to report the delay it is running at.
///
/// Exact, unlike `wait_for_text`, which asks whether the page *contains* a string: now that
/// the default is 50, "delay = 5" is a substring of "delay = 50" and "delay = 500" - so
/// waiting on the short one by text would pass while a longer one was still on screen.
async fn wait_for_delay(client: &Client, expected: u32) {
    const SCRIPT: &str = r#"
        const node = Array.from(document.querySelectorAll('div'))
            .find((candidate) => /^delay = \d+$/.test(candidate.textContent.trim()));
        return node === undefined ? null : parseInt(node.textContent.trim().slice(8), 10);
    "#;

    wait_until(&format!("the delay to become {expected}"), || async {
        let shown = client
            .execute(SCRIPT, vec![])
            .await
            .ok()
            .and_then(|value| value.as_u64());

        Ok(shown == Some(expected as u64))
    })
    .await;
}

/// The generation counter the Game Of Life board is showing.
///
/// Matched on the whole label rather than with `wait_for_text`, which asks whether the page
/// *contains* a string: "Year = 1" is a substring of "Year = 10", so waiting on one of those
/// by text starts answering the wrong question as soon as the board passes its ninth
/// generation - which at a five millisecond delay is immediately.
async fn life_year(client: &Client) -> u32 {
    const SCRIPT: &str = r#"
        const node = Array.from(document.querySelectorAll('div'))
            .find((candidate) => /^Year = \d+$/.test(candidate.textContent.trim()));
        return node === undefined ? null : parseInt(node.textContent.trim().slice(7), 10);
    "#;

    client
        .execute(SCRIPT, vec![])
        .await
        .ok()
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| panic!("could not read the Game Of Life generation counter")) as u32
}

/// Press a preset button and wait for its cells to land.
///
/// Loading also resets the generation counter, so this is where a preset run starts.
async fn load_pattern(client: &Client, name: &str, population: usize) {
    click_by_text(client, "button", name).await;

    wait_until(&format!("{name} to put {population} cells on the board"), {
        || async { Ok(life_population(client).await == population && life_year(client).await == 1) }
    })
    .await;
}

/// Press Step `times` times.
///
/// Driven from one script rather than one WebDriver click each. It is the same button and the
/// same handler, but a round trip per press turns the hundred and thirty generations Diehard
/// needs into several seconds of the run. The click is a real one - vertigo attaches its
/// handlers with `addEventListener`, so nothing here bypasses the app.
async fn step_times(client: &Client, times: usize) {
    let script = format!(
        "const step = Array.from(document.querySelectorAll('button'))\
           .find((node) => node.textContent.trim() === 'Step');\
         if (!step) {{ return 'no Step button'; }}\
         for (let n = 0; n < {times}; n += 1) {{ step.click(); }}\
         return null;"
    );

    let complaint = client
        .execute(&script, vec![])
        .await
        .expect("stepping the board failed");

    assert!(
        complaint.is_null(),
        "could not press Step {times} times: {complaint}"
    );
}

/// The x position of the element matching `selector` whose own text is `text`.
///
/// Rounded, because a sub-pixel difference is not the kind of movement worth failing over.
async fn element_left(client: &Client, selector: &str, text: &str) -> i64 {
    let (left, _, _, _) = find_by_text(client, selector, text)
        .await
        .rectangle()
        .await
        .unwrap_or_else(|err| panic!("reading the position of the {selector} {text:?}: {err}"));

    left.round() as i64
}

/// How many cells the Game Of Life board says are alive.
async fn life_population(client: &Client) -> usize {
    const SCRIPT: &str = r#"
        const node = Array.from(document.querySelectorAll('div'))
            .find((candidate) => /^Population = \d+$/.test(candidate.textContent.trim()));
        return node === undefined ? null : parseInt(node.textContent.trim().slice(13), 10);
    "#;

    client
        .execute(SCRIPT, vec![])
        .await
        .ok()
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| panic!("could not read the Game Of Life population")) as usize
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
