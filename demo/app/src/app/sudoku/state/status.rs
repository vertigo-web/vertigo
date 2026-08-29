//! Whether the board as it stands is empty, going somewhere, broken, or done.
//!
//! One `Computed` reading all 81 cells - the widest fan-in in the demo, and the counterpart to
//! `possible_values`, which fans the other way.

use std::collections::HashSet;

use vertigo::Computed;

use super::Cell;
use super::number_item::SudokuValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Empty,
    InProgress {
        filled: usize,
    },
    /// A digit appears twice in some row, column or block.
    Conflict,
    Solved,
}

impl Status {
    pub fn message(&self) -> String {
        match self {
            Status::Empty => "Empty board".to_string(),
            Status::InProgress { filled } => format!("{filled} of 81 filled"),
            Status::Conflict => "Conflict: a digit repeats".to_string(),
            Status::Solved => "Solved!".to_string(),
        }
    }

    pub fn colour(&self) -> &'static str {
        match self {
            Status::Conflict => "#b00",
            Status::Solved => "#070",
            _ => "#555",
        }
    }
}

type Board = [[Option<SudokuValue>; 9]; 9];

/// `rows` is the grid in reading order - see `SudokuState::rows`.
pub fn status(rows: Vec<Vec<Cell>>) -> Computed<Status> {
    Computed::from(move |context| {
        let mut board: Board = [[None; 9]; 9];
        let mut filled = 0;

        for (row_n, row) in rows.iter().enumerate() {
            for (col_n, cell) in row.iter().enumerate() {
                let value = cell.number.get(context);

                if value.is_some() {
                    filled += 1;
                }

                board[row_n][col_n] = value;
            }
        }

        if has_conflict(&board) {
            Status::Conflict
        } else if filled == 81 {
            Status::Solved
        } else if filled == 0 {
            Status::Empty
        } else {
            Status::InProgress { filled }
        }
    })
}

fn has_conflict(board: &Board) -> bool {
    (0..9).any(|n| {
        let block_row = 3 * (n / 3);
        let block_col = 3 * (n % 3);

        repeats(board[n].iter().copied())
            || repeats((0..9).map(|row| board[row][n]))
            || repeats((0..9).map(|cell| board[block_row + cell / 3][block_col + cell % 3]))
    })
}

fn repeats(unit: impl Iterator<Item = Option<SudokuValue>>) -> bool {
    let mut seen = HashSet::new();
    unit.flatten().any(|value| !seen.insert(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(rows: [&str; 9]) -> Board {
        let mut board: Board = [[None; 9]; 9];

        for (row_n, row) in rows.iter().enumerate() {
            for (col_n, byte) in row.bytes().enumerate() {
                board[row_n][col_n] = SudokuValue::from_ascii(byte);
            }
        }

        board
    }

    #[test]
    fn an_empty_board_has_no_conflict() {
        assert!(!has_conflict(&parse(["........."; 9])));
    }

    #[test]
    fn a_repeat_in_a_row_is_a_conflict() {
        assert!(has_conflict(&parse([
            "1..1.....",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
        ])));
    }

    #[test]
    fn a_repeat_in_a_column_is_a_conflict() {
        assert!(has_conflict(&parse([
            "....1....",
            ".........",
            ".........",
            "....1....",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
        ])));
    }

    /// Same 3x3 block, but a different row and a different column - so neither of the checks
    /// above would see it.
    #[test]
    fn a_repeat_in_a_block_is_a_conflict() {
        assert!(has_conflict(&parse([
            "......7..",
            ".......7.",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
            ".........",
        ])));
    }

    /// The solution to `examples::EASY`, which must read as clean.
    #[test]
    fn a_finished_board_has_no_conflict() {
        assert!(!has_conflict(&parse([
            "435269781",
            "682571493",
            "197834562",
            "826195347",
            "374682915",
            "951743628",
            "519326874",
            "248957136",
            "763418259",
        ])));
    }
}
