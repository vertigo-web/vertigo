use vertigo::{Computed, Value, get_driver};

use self::{
    number_item::{NumberItem, SudokuValue},
    possible_values::{PossibleValues, possible_values},
    possible_values_last::{PossibleValuesLast, possible_values_last},
    sudoku_square::SudokuSquare,
    tree_box::TreeBoxIndex,
};

pub mod examples;
pub mod number_item;
pub mod possible_values;
pub mod possible_values_last;
pub mod status;
pub mod sudoku_square;
pub mod tree_box;

use examples::Board;
pub use status::Status;

fn create_grid() -> SudokuSquare<SudokuSquare<NumberItem>> {
    SudokuSquare::create_with_iterator(move |_level0x, _level0y| {
        SudokuSquare::create_with_iterator(move |_level1x, _level1y| NumberItem::new(None))
    })
}

fn create_grid_possible(
    grid_number: &SudokuSquare<SudokuSquare<NumberItem>>,
) -> SudokuSquare<SudokuSquare<PossibleValues>> {
    SudokuSquare::create_with_iterator(|level0x, level0y| {
        SudokuSquare::create_with_iterator(|level1x, level1y| {
            possible_values(grid_number, level0x, level0y, level1x, level1y)
        })
    })
}

fn create_grid_possible_last(
    grid_number: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid_possible: &SudokuSquare<SudokuSquare<PossibleValues>>,
) -> SudokuSquare<SudokuSquare<PossibleValuesLast>> {
    SudokuSquare::create_with_iterator(|level0x, level0y| {
        SudokuSquare::create_with_iterator(|level1x, level1y| {
            possible_values_last(
                grid_number,
                grid_possible,
                level0x,
                level0y,
                level1x,
                level1y,
            )
        })
    })
}

#[derive(Clone)]
pub struct Cell {
    pub number: NumberItem,
    pub possible: PossibleValues,
    pub possible_last: PossibleValuesLast,
    /// Part of the puzzle as loaded, rather than something the player put there. Givens are
    /// styled differently and cannot be deleted.
    pub is_given: Value<bool>,
}

fn create_grid_view(
    grid_number: SudokuSquare<SudokuSquare<NumberItem>>,
    grid_possible: SudokuSquare<SudokuSquare<PossibleValues>>,
    grid_possible_last: SudokuSquare<SudokuSquare<PossibleValuesLast>>,
) -> SudokuSquare<SudokuSquare<Cell>> {
    SudokuSquare::create_with_iterator(|level0x, level0y| {
        SudokuSquare::create_with_iterator(|level1x, level1y| {
            let number = grid_number
                .get_from(level0x, level0y)
                .get_from(level1x, level1y);
            let possible = grid_possible
                .get_from(level0x, level0y)
                .get_from(level1x, level1y);
            let possible_last = grid_possible_last
                .get_from(level0x, level0y)
                .get_from(level1x, level1y);

            Cell {
                number: number.clone(),
                possible: possible.clone(),
                possible_last: possible_last.clone(),
                is_given: Value::new(false),
            }
        })
    })
}

#[derive(Clone)]
pub struct SudokuState {
    pub grid: SudokuSquare<SudokuSquare<Cell>>,
    /// Empty, in progress, broken or done - derived from all 81 cells.
    pub status: Computed<Status>,
    /// Whether empty cells show what the solver has worked out.
    ///
    /// Off by default: with them on, the two rules in `possible_values` and
    /// `possible_values_last` settle most of a board on their own, and there is not much of a
    /// puzzle left to do.
    pub hints: Value<bool>,
}

impl SudokuState {
    pub fn new() -> Self {
        let grid_number = create_grid();
        let grid_possible = create_grid_possible(&grid_number);
        let grid_possible_last = create_grid_possible_last(&grid_number, &grid_possible);

        let grid = create_grid_view(grid_number, grid_possible, grid_possible_last);
        let status = status::status(rows_of(&grid));

        Self {
            grid,
            status,
            hints: Value::new(false),
        }
    }

    /// The grid as nine rows of nine cells, top row first.
    fn rows(&self) -> Vec<Vec<Cell>> {
        rows_of(&self.grid)
    }

    pub fn clear(&self) {
        log::info!("clear");

        get_driver().transaction(|_| {
            for cell in self.rows().into_iter().flatten() {
                cell.number.set(None);
                cell.is_given.set(false);
            }
        });
    }

    pub fn example1(&self) {
        self.load(&examples::EASY, "Easy");
    }

    pub fn example2(&self) {
        self.load(&examples::MEDIUM, "Medium");
    }

    pub fn example3(&self) {
        self.load(&examples::HARD, "Hard");
    }

    /// Put `board` on the grid, replacing whatever was there.
    ///
    /// Every one of the 81 cells is written, blanks included, so this is a load and a clear at
    /// once. One transaction, because each write fans out to the cell's peers through
    /// `possible_values` and `possible_values_last`.
    fn load(&self, board: &Board, name: &str) {
        log::info!("loading {name}");

        get_driver().transaction(|_| {
            for (row, cells) in self.rows().into_iter().enumerate() {
                let line = board[row].as_bytes();

                for (col, cell) in cells.into_iter().enumerate() {
                    let value = line.get(col).copied().and_then(SudokuValue::from_ascii);

                    cell.number.set(value);
                    cell.is_given.set(value.is_some());
                }
            }
        });
    }
}

/// The grid in reading order.
///
/// `get_from` takes the row first and the column second at both levels, which is the order
/// `MainRender` lays the nine blocks out in.
fn rows_of(grid: &SudokuSquare<SudokuSquare<Cell>>) -> Vec<Vec<Cell>> {
    let mut rows = Vec::with_capacity(9);

    for block_row in TreeBoxIndex::variants() {
        for in_row in TreeBoxIndex::variants() {
            let mut row = Vec::with_capacity(9);

            for block_col in TreeBoxIndex::variants() {
                for in_col in TreeBoxIndex::variants() {
                    row.push(
                        grid.get_from(block_row, block_col)
                            .get_from(in_row, in_col)
                            .clone(),
                    );
                }
            }

            rows.push(row);
        }
    }

    rows
}
