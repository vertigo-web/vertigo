//! The page bodies.
//!
//! Every one of these returns one [`wrapper`] containing N of exactly one thing, and
//! `/tiny` returns the wrapper empty - which is what makes `/tiny` an exact subtrahend for
//! all the others.
//!
//! Built with `DomElement` rather than `dom!` because the shapes are parametric: `dom!`
//! wants the tree written out, and a benchmark wants `WIDE_N` to be one constant rather
//! than five thousand lines. The commands that reach the server are identical either way.

use vertigo::{Css, DomElement, DomNode, get_driver};

use crate::shapes::{
    ATTR_N, ATTRS_PER, CSS_N, DEEP_D, ESCAPE_EVERY, ROUNDTRIP_N, TABLE_COLS, TABLE_ROWS, TEXT_LEN,
    TEXT_N, WIDE_N,
};

/// The wrapper every page puts its units inside, and the only thing `/tiny` renders.
///
/// Always exactly one element with exactly one attribute, whatever the tag - three DOM
/// commands - so `commands(page) − commands(/tiny)` is exactly `N × per_unit` with no
/// remainder to explain. Getting this wrong is not a small thing: with a wrapper that
/// differed between pages the per-unit figures came out as 4.9998 rather than 5, and the
/// assertions would have had to be written as approximations.
fn wrapper(tag: &'static str) -> DomElement {
    DomElement::new(tag).attr("class", "unit")
}

/// The wrapper and nothing in it. The floor every other page is read against.
pub fn tiny() -> DomNode {
    wrapper("div").into()
}

/// `WIDE_N` siblings under one parent.
///
/// Stresses the host's `feed`: one `HashMap` insert per node into `AllElements`, and one
/// splice into the circular child list, `WIDE_N` times against a list that keeps growing.
/// The text is fixed-width so the page's byte size is exactly linear in `WIDE_N`.
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

/// One chain `DEEP_D` levels deep, inside `<pre>`.
///
/// Stresses recursion - `get_response_one_elements` and `html_node_to_string` both descend
/// once per level - at a node count comparable to a flat page. `<pre>` turns the
/// pretty-printer's indentation off for the whole subtree, which is the point: this page
/// and `/deep-indent` are the same tree, and the difference between them is *only*
/// whitespace.
pub fn deep() -> DomNode {
    wrapper("pre").child(deep_chain()).into()
}

/// The same chain without `<pre>`, so the pretty-printer indents every level.
///
/// `Format::get` is `" ".repeat(depth * 2)` per node, so the whitespace alone is O(DEEP_D²):
/// a 17 KiB page serialises to over four megabytes at 1 500 levels. Reading this against
/// `/deep` prices the pretty-printer on its own.
pub fn deep_indent() -> DomNode {
    wrapper("div").child(deep_chain()).into()
}

/// Built innermost-first, because each level has to own the one below it.
///
/// The innermost `<div>` is left empty rather than given a text leaf: a leaf would be four
/// commands that belong to no level, and `/deep` would cost `2 × DEEP_D + 4` instead of a
/// clean multiple.
fn deep_chain() -> DomNode {
    let mut node: DomNode = DomElement::new("div").into();

    for _ in 1..DEEP_D {
        node = DomElement::new("div").child(node).into();
    }

    node
}

/// `TEXT_N` paragraphs whose text needs escaping.
pub fn text() -> DomNode {
    paragraphs(true)
}

/// The same paragraphs with nothing to escape.
///
/// Identical command stream to `/text` - same nodes, same text lengths - so the difference
/// between the two is `encode_safe` doing work rather than passing a `Cow::Borrowed`
/// through.
pub fn text_plain() -> DomNode {
    paragraphs(false)
}

fn paragraphs(escapable: bool) -> DomNode {
    let parent = wrapper("div");

    for index in 0..TEXT_N {
        parent.add_child(DomElement::new("p").child_text(filler(index, escapable)));
    }

    parent.into()
}

/// `TEXT_LEN` characters, deterministically. Every `ESCAPE_EVERY`th is one `encode_safe`
/// has to replace, when asked for.
fn filler(seed: u32, escapable: bool) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz ";
    const ESCAPES: [char; 4] = ['<', '>', '&', '"'];

    let mut out = String::with_capacity(TEXT_LEN);

    for position in 0..TEXT_LEN {
        if escapable && position % ESCAPE_EVERY == ESCAPE_EVERY - 1 {
            out.push(ESCAPES[position / ESCAPE_EVERY % ESCAPES.len()]);
        } else {
            let index = (position + seed as usize) % ALPHABET.len();
            out.push(ALPHABET[index] as char);
        }
    }

    out
}

/// `ATTR_N` elements carrying `ATTRS_PER` attributes each.
///
/// Attributes are the one part of a node that is paid for three times: a `BTreeMap` insert
/// in `feed`, a whole-map `clone` in `get_response_one_elements`, and
/// `encode_quoted_attribute` in the serialiser. A page that is mostly attributes says which
/// of the three moved.
pub fn attrs() -> DomNode {
    let parent = wrapper("div");

    for index in 0..ATTR_N {
        let mut element = DomElement::new("div").attr("class", "a");

        for slot in 0..ATTRS_PER {
            element = element.attr(
                format!("data-a{slot:02}"),
                format!("{index:08}-{slot:02}---"),
            );
        }

        parent.add_child(element.child_text("x"));
    }

    parent.into()
}

/// `CSS_N` distinct rules, one per element.
///
/// `css!` takes a literal, so it cannot express "three hundred different rules". `Css::string`
/// goes down `CssManager::get_dynamic`, which is the path an app with computed styles takes
/// anyway. One flat declaration block per rule and no `:hover`/`@media`, so each produces
/// exactly one selector and the count stays a constant times `CSS_N`.
pub fn css() -> DomNode {
    let parent = wrapper("div");

    for index in 0..CSS_N {
        let rule = Css::string(format!(
            "color: #{:06x}; padding: {}px; margin: {}px",
            index * 1_117 % 0xff_ffff,
            index % 17,
            index % 7
        ));

        parent.add_child(DomElement::new("div").css(rule).child_text("styled"));
    }

    parent.into()
}

/// `TABLE_ROWS` × `TABLE_COLS`, the js-framework-benchmark shape.
///
/// The one synthetic page that is a mixture rather than an isolate: a realistic ratio of
/// elements to text to attributes, for a number that can be sanity-checked against what
/// other frameworks report.
pub fn table() -> DomNode {
    let table = wrapper("table");

    for row_index in 0..TABLE_ROWS {
        let row = DomElement::new("tr").attr("class", "row");

        for column in 0..TABLE_COLS {
            row.add_child(DomElement::new("td").child_text(format!("{row_index}-{column}")));
        }

        table.add_child(row);
    }

    table.into()
}

/// `ROUNDTRIP_N` crossings of the wasm/host boundary that emit no DOM at all.
///
/// `is_browser()` is not cached - every call is a full `exec_command`: a `JsJson` encode in
/// the guest, a read out of linear memory on the host, a re-encode of the answer, and a
/// guest allocation for it. Real pages make a lot of these without meaning to, and inside
/// "time spent in wasm" they are invisible. Here they are the entire page.
///
/// The result is folded into the rendered text so neither LLVM nor `wasm-opt -Os` has a
/// reason to delete the loop.
pub fn roundtrip() -> DomNode {
    let driver = get_driver();
    let mut crossings: u32 = 0;

    for _ in 0..ROUNDTRIP_N {
        if driver.is_browser() {
            crossings += 2;
        } else {
            crossings += 1;
        }
    }

    // Folded into the wrapper's own text rather than into a node of its own, so the page
    // costs the wrapper plus one text node however large `ROUNDTRIP_N` gets. Whatever this
    // route is expensive for, it is not DOM work.
    wrapper("div")
        .child_text(format!("crossings={crossings}"))
        .into()
}
