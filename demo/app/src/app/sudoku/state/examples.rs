//! The boards behind the three example buttons.
//!
//! Each is written as nine rows of nine characters, `.` for a blank, so that the constant
//! reads the way the board renders. All three are valid and have exactly one solution.
//!
//! They differ in how far the demo's own hints carry you, which is the point of having three:
//!
//!  * `EASY` and `MEDIUM` are settled entirely by the two rules the demo already implements -
//!    a cell with one remaining candidate (`possible_values`) and a candidate with one
//!    remaining home in its row, column or block (`possible_values_last`). Clicking the hints
//!    the board offers is enough to finish either one.
//!  * `HARD` is not. Those two rules fill 25 of its 81 cells and then stop, so the board
//!    stalls with several candidates showing everywhere - which is the honest picture of what
//!    this solver does and does not do.

/// A board, top row first. Nine rows of nine bytes each.
pub type Board = [&'static str; 9];

pub const EASY: Board = [
    "...26.7.1",
    "68..7..9.",
    "19...45..",
    "82.1...4.",
    "..46.29..",
    ".5...3.28",
    "..93...74",
    ".4..5..36",
    "7.3.18...",
];

pub const MEDIUM: Board = [
    "53..7....",
    "6..195...",
    ".98....6.",
    "8...6...3",
    "4..8.3..1",
    "7...2...6",
    ".6....28.",
    "...419..5",
    "....8..79",
];

pub const HARD: Board = [
    "..53.....",
    "8......2.",
    ".7..1.5..",
    "4....53..",
    ".1..7...6",
    "..32...8.",
    ".6.5....9",
    "..4....3.",
    ".....97..",
];
