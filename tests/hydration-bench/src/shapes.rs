//! How big each page is.
//!
//! Smaller than the SSR benchmark's equivalents on purpose. There every route was rendered
//! once per sample inside one process; here every sample is a full browser page load, and
//! the JS implementation's vnode fold is quadratic in page size - `/wide` at the SSR
//! benchmark's 5 000 would put a single load into seconds and the suite into tens of
//! minutes.
//!
//! Mirrored in `tests.rs`, which uses them to size the mutation-count assertions. They are
//! imported from here rather than copied, because the driver and this app share a
//! compilation.

/// Sibling `<div>`s under one parent.
pub const WIDE_N: u32 = 1_500;

/// Nesting depth for `/deep`.
///
/// **Must stay below 512.** Chrome's HTML parser caps element nesting at 512 and flattens
/// whatever is deeper, so a server document nested past that is not the document the browser
/// ends up holding - hydration then cannot match nodes that were never built. At 600 this
/// page reported 516 of 607 matched and looked like an implementation defect; it was the
/// parser. 400 leaves room without being near the edge.
pub const DEEP_D: u32 = 400;

/// Paragraphs on `/text`.
pub const TEXT_N: u32 = 300;
/// Characters in each paragraph.
pub const TEXT_LEN: usize = 400;

/// Elements on `/attrs`.
pub const ATTR_N: u32 = 500;
/// Attributes on each of them, over and above the `class`.
pub const ATTRS_PER: u32 = 12;

/// Rows and cells on `/table`.
pub const TABLE_ROWS: u32 = 400;
pub const TABLE_COLS: u32 = 8;

/// Rows on each of the four `/mismatch-*` pages.
///
/// The same count on all of them, so the four are directly comparable to each other and the
/// only thing that varies is *how* the browser's tree differs from the server's.
pub const MISMATCH_N: u32 = 500;
