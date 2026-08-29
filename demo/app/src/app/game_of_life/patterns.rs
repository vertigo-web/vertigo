//! Starting boards for the Game Of Life tab.
//!
//! Each is written as rows of `#` for a live cell, so the constant looks like what it draws.

use std::collections::HashSet;

pub struct Pattern {
    pub name: &'static str,
    /// What it does, shown as the button's tooltip.
    pub about: &'static str,
    pub cells: &'static [&'static str],
    /// Top-left corner on the board, as `(row, column)`. `None` centres it.
    pub at: Option<(u16, u16)>,
}

impl Pattern {
    /// The board coordinates this pattern occupies, wrapped into a `height` x `width` torus.
    pub fn live_cells(&self, width: u16, height: u16) -> HashSet<(u16, u16)> {
        let rows = self.cells.len() as u16;
        let cols = self.cells.iter().map(|row| row.len()).max().unwrap_or(0) as u16;

        let (top, left) = self
            .at
            .unwrap_or_else(|| ((height - rows) / 2, (width - cols) / 2));

        let mut live = HashSet::new();

        for (row_n, row) in self.cells.iter().enumerate() {
            for (col_n, byte) in row.bytes().enumerate() {
                if byte == b'#' {
                    let y = (top + row_n as u16) % height;
                    let x = (left + col_n as u16) % width;
                    live.insert((y, x));
                }
            }
        }

        live
    }
}

/// The presets, in the order the buttons appear.
pub const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "Glider",
        about: "Travels one cell diagonally every four generations",
        cells: &[".#.", "..#", "###"],
        // Near a corner, so it has the whole board to cross.
        at: Some((4, 4)),
    },
    Pattern {
        name: "Spaceship",
        about: "Lightweight spaceship: two cells sideways every four generations",
        cells: &[".####", "#...#", "....#", "#..#."],
        at: Some((14, 4)),
    },
    Pattern {
        name: "Pulsar",
        about: "An oscillator with a period of three",
        cells: &[
            "..###...###..",
            ".............",
            "#....#.#....#",
            "#....#.#....#",
            "#....#.#....#",
            "..###...###..",
            ".............",
            "..###...###..",
            "#....#.#....#",
            "#....#.#....#",
            "#....#.#....#",
            ".............",
            "..###...###..",
        ],
        at: None,
    },
    Pattern {
        name: "Glider gun",
        about: "Gosper's glider gun: emits a glider every thirty generations, forever",
        cells: &[
            "........................#...........",
            "......................#.#...........",
            "............##......##............##",
            "...........#...#....##............##",
            "##........#.....#...##..............",
            "##........#...#.##....#.#...........",
            "..........#.....#.......#...........",
            "...........#...#....................",
            "............##......................",
        ],
        // Top-left, so the gliders it fires have room to run.
        at: Some((2, 2)),
    },
    Pattern {
        name: "Twin guns",
        about: "Two Gosper guns facing each other, their glider streams colliding in the middle",
        cells: &[
            "........................#........................................#........................",
            "......................#.#........................................#.#......................",
            "............##......##............##..................##............##......##............",
            "...........#...#....##............##..................##............##....#...#...........",
            "##........#.....#...##..............................................##...#.....#........##",
            "##........#...#.##....#.#........................................#.#....##.#...#........##",
            "..........#.....#.......#........................................#.......#.....#..........",
            "...........#...#..........................................................#...#...........",
            "............##..............................................................##............",
        ],
        at: Some((2, 15)),
    },
    Pattern {
        name: "Menagerie",
        about: "Still lifes and oscillators side by side, none of them touching",
        cells: &[
            "##.........##........##.......##.........#................",
            "##........#..#......#..#......#.#.......#.#...............",
            "...........##........#.#.......#.........#................",
            "......................#...................................",
            "..........................................................",
            "..........................................................",
            "..........................................................",
            ".###.....###....##.........................###...###......",
            "........###.....##.........#....#.........................",
            "..................##.....##.####.##......#....#.#....#....",
            "..................##.......#....#........#....#.#....#....",
            ".........................................#....#.#....#....",
            "...........................................###...###......",
            "..........................................................",
            "...........................................###...###......",
            ".........................................#....#.#....#....",
            ".........................................#....#.#....#....",
            ".........................................#....#.#....#....",
            "..........................................................",
            "...........................................###...###......",
        ],
        at: None,
    },
    Pattern {
        name: "Pentadecathlon",
        about: "An oscillator with a period of fifteen",
        cells: &["..#....#..", "##.####.##", "..#....#.."],
        at: None,
    },
    Pattern {
        name: "R-pentomino",
        about: "Five cells that take over a thousand generations to settle down",
        cells: &[".##", "##.", ".#."],
        at: None,
    },
    Pattern {
        name: "Acorn",
        about: "Seven cells that grow into several hundred",
        cells: &[".#.....", "...#...", "##..###"],
        at: None,
    },
    Pattern {
        name: "Diehard",
        about: "Seven cells that vanish without trace, at generation 130 exactly",
        cells: &["......#.", "##......", ".#...###"],
        at: None,
    },
];
