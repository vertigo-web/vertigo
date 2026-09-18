//! The page bodies.
//!
//! Every page returns one wrapper holding N units of one shape, so `/tiny` - the wrapper
//! with nothing in it - is the fixed cost every other page also pays.
//!
//! The four `/mismatch-*` pages render a *different tree* in the browser from the one the
//! server sent. That is the whole point of them: a page that matches perfectly says what
//! adopting costs, and a page that does not says what disagreeing costs. They branch on
//! `get_driver().is_browser()`, which is how `demo/app/src/app/driver/ssr_test.rs` already
//! stages the same situation.

use vertigo::{DomElement, DomNode, get_driver};

use crate::shapes::{
    ATTR_N, ATTRS_PER, DEEP_D, MISMATCH_N, TABLE_COLS, TABLE_ROWS, TEXT_LEN, TEXT_N, WIDE_N,
};

/// One element, one attribute, whatever the tag - so the wrapper costs the same on every
/// page and `/tiny` subtracts cleanly.
fn wrapper(tag: &'static str) -> DomElement {
    DomElement::new(tag).attr("class", "unit")
}

/// The wrapper and nothing in it.
pub fn tiny() -> DomNode {
    wrapper("div").into()
}

// ---------------------------------------------------------------------------------------
// clean match - server and browser render the same tree
// ---------------------------------------------------------------------------------------

/// `WIDE_N` siblings under one parent. The flat case, and the one where the old
/// implementation's per-`InsertBefore` rescan of every vnode costs the most.
pub fn wide() -> DomNode {
    let parent = wrapper("div");

    for index in 0..WIDE_N {
        parent.add_child(
            DomElement::new("div")
                .attr("class", "r")
                .child_text(format!("{index:04}")),
        );
    }

    parent.into()
}

/// One chain `DEEP_D` levels deep. The recursive case: both matchers descend once per level.
pub fn deep() -> DomNode {
    let mut node: DomNode = DomElement::new("div").into();

    for _ in 1..DEEP_D {
        node = DomElement::new("div").child(node).into();
    }

    wrapper("div").child(node).into()
}

/// `TEXT_N` paragraphs. Text nodes are matched by content rather than by tag, so this is the
/// other half of the matcher.
pub fn text() -> DomNode {
    let parent = wrapper("div");

    for index in 0..TEXT_N {
        parent.add_child(DomElement::new("p").child_text(filler(index)));
    }

    parent.into()
}

/// `ATTR_N` elements carrying `ATTRS_PER` attributes each.
///
/// Attribute reconciliation is bidirectional in both implementations - every attribute the
/// server sent has to be checked against the one the browser wants, in both directions - so
/// a page that is mostly attributes isolates that pass.
pub fn attrs() -> DomNode {
    let parent = wrapper("div");

    for index in 0..ATTR_N {
        let mut element = DomElement::new("div").attr("class", "a");

        for slot in 0..ATTRS_PER {
            element = element.attr(format!("data-a{slot:02}"), format!("{index:06}-{slot:02}"));
        }

        parent.add_child(element.child_text("x"));
    }

    parent.into()
}

/// `TABLE_ROWS` × `TABLE_COLS`. A realistic mixture rather than an isolate.
///
/// The `<tbody>` is not optional and is not cosmetic. A `<tr>` written directly inside
/// `<table>` is invalid HTML, and the browser's parser silently inserts a `<tbody>` around
/// it. The parsed server document is then `table > tbody > tr` while the application's tree
/// is `table > tr`; they disagree at the first child, and nothing below the table matches.
/// Written without it, this page hydrated 6 of 6 807 nodes.
pub fn table() -> DomNode {
    let table = wrapper("table");
    let body = DomElement::new("tbody");

    for row_index in 0..TABLE_ROWS {
        let row = DomElement::new("tr").attr("class", "row");

        for column in 0..TABLE_COLS {
            row.add_child(DomElement::new("td").child_text(format!("{row_index}-{column}")));
        }

        body.add_child(row);
    }

    table.child(body).into()
}

// ---------------------------------------------------------------------------------------
// mismatch - the browser's tree differs from the server's
// ---------------------------------------------------------------------------------------

/// Adjacent pairs of rows are swapped in the browser.
///
/// A pairwise swap rather than a full reversal on purpose. Both implementations match
/// positionally with a forward scan, so a reversal makes every row a miss and turns the page
/// into a quadratic worst case that says more about the size of `MISMATCH_N` than about the
/// algorithm. A swap is what a real reorder looks like and still defeats naive positional
/// matching at every single position.
///
/// Whatever `MISMATCH_N` is, what the browser renders is a **permutation** of what the server
/// sent - the same rows, in a different order, and nothing else. That is what makes this page
/// price reordering rather than reordering plus a rebuild.
pub fn mismatch_order() -> DomNode {
    let parent = wrapper("div");
    let swap = get_driver().is_browser();

    for index in 0..MISMATCH_N {
        let shown = match swap {
            false => index,
            // An odd-length list leaves its last row without a partner, so it stays where it
            // is. Swapping it anyway would map it to `MISMATCH_N` - a row the server never
            // rendered - which makes the page a *content* mismatch as well as an order one,
            // and the mutation counts it produces would no longer be attributable to
            // reordering alone.
            true if index % 2 == 0 && index + 1 == MISMATCH_N => index,
            true if index % 2 == 0 => index + 1,
            true => index - 1,
        };

        parent.add_child(row(shown));
    }

    parent.into()
}

/// The browser wraps every row in two levels the server never sent.
pub fn mismatch_depth() -> DomNode {
    let parent = wrapper("div");
    let deeper = get_driver().is_browser();

    for index in 0..MISMATCH_N {
        if deeper {
            parent.add_child(
                DomElement::new("div").attr("class", "outer").child(
                    DomElement::new("div")
                        .attr("class", "inner")
                        .child(row(index)),
                ),
            );
        } else {
            parent.add_child(row(index));
        }
    }

    parent.into()
}

/// The server puts an attribute on every row that the browser does not want, so every row
/// needs a `RemoveAttr` and nothing needs rebuilding.
pub fn mismatch_attrs() -> DomNode {
    let parent = wrapper("div");
    let server = !get_driver().is_browser();

    for index in 0..MISMATCH_N {
        let mut element = DomElement::new("div").attr("class", "r");

        if server {
            element = element
                .attr("data-server-only", "1")
                .attr("title", "rendered on the server");
        }

        parent.add_child(element.child_text(format!("{index:04}")));
    }

    parent.into()
}

/// Every row is three adjacent text nodes, which the server glues into one.
///
/// This is the case the old implementation compensates for with a "skip the rest of the run"
/// flag that counts the skipped nodes as matched without binding them to anything, and which
/// the new one is claimed to fix by splitting the run and keeping every node bound to its id.
/// The mutation counts here are where that difference shows up.
pub fn mismatch_text() -> DomNode {
    let parent = wrapper("div");

    for index in 0..MISMATCH_N {
        parent.add_child(
            DomElement::new("p")
                .child_text(format!("{index:04}"))
                .child_text("-")
                .child_text("tail"),
        );
    }

    parent.into()
}

// ---------------------------------------------------------------------------------------

fn row(index: u32) -> DomElement {
    DomElement::new("div")
        .attr("class", "r")
        .child_text(format!("{index:04}"))
}

/// `TEXT_LEN` characters, deterministically. Nothing needing HTML escaping, and nothing from
/// the clock or the RNG - every load of a route has to produce byte-identical markup or the
/// samples are not samples of the same thing.
fn filler(seed: u32) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz ";

    let mut out = String::with_capacity(TEXT_LEN);

    for position in 0..TEXT_LEN {
        let index = (position + seed as usize) % ALPHABET.len();
        out.push(ALPHABET[index] as char);
    }

    out
}
