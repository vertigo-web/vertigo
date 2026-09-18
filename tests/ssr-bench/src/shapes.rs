//! How big each page is.
//!
//! One place, because `tests.rs` mirrors these to compute the expected command counts. The
//! `dom-bench` suite sets the precedent: the driver duplicates the scene sizes and the
//! assertions fail loudly if the two drift.
//!
//! Sized so a release render lands in the low milliseconds - big enough that a phase is
//! well clear of clock noise, small enough that eleven routes times a few dozen samples is
//! a run you will actually wait for.

/// Sibling `<div>`s under one parent.
pub const WIDE_N: u32 = 5_000;

/// Nesting depth for `/deep` and `/deep-indent`.
///
/// Both `AllElements::get_response` and `html_node_to_string` recurse once per level, so
/// this is bounded by the test thread's stack rather than by patience. 1 500 is
/// comfortable; do not reach for 50 000 without measuring the stack first.
pub const DEEP_D: u32 = 1_500;

/// Paragraphs on `/text` and `/text-plain`.
pub const TEXT_N: u32 = 400;
/// Characters in each of those paragraphs.
pub const TEXT_LEN: usize = 2_000;
/// On `/text`, every Nth character is one that `encode_safe` has to replace.
pub const ESCAPE_EVERY: usize = 64;

/// Elements on `/attrs`.
pub const ATTR_N: u32 = 1_000;
/// Attributes on each of them, over and above the `class`.
pub const ATTRS_PER: u32 = 12;

/// Distinct dynamic css rules on `/css`.
pub const CSS_N: u32 = 300;

/// Rows and cells on `/table`.
pub const TABLE_ROWS: u32 = 1_000;
pub const TABLE_COLS: u32 = 10;

/// Driver round trips on `/roundtrip`.
pub const ROUNDTRIP_N: u32 = 500;

/// The body `/plain.txt` answers with, before any of the tree it built is serialised.
pub const PLAIN_BODY: &str = "ssr-bench plain text route\n";
