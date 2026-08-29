use vertigo::{Value, get_driver};

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
pub mod sudoku_square;
pub mod tree_box;

use examples::Board;

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
    pub show_delete: Value<bool>,
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
                show_delete: Value::new(true),
            }
        })
    })
}

#[derive(Clone)]
pub struct SudokuState {
    pub grid: SudokuSquare<SudokuSquare<Cell>>,
}

impl SudokuState {
    pub fn new() -> Self {
        let grid_number = create_grid();
        let grid_possible = create_grid_possible(&grid_number);
        let grid_possible_last = create_grid_possible_last(&grid_number, &grid_possible);

        Self {
            grid: create_grid_view(grid_number, grid_possible, grid_possible_last),
        }
    }

    pub fn clear(&self) {
        log::info!("clear");

        get_driver().transaction(|_| {
            for x0 in TreeBoxIndex::variants() {
                for y0 in TreeBoxIndex::variants() {
                    for x1 in TreeBoxIndex::variants() {
                        for y1 in TreeBoxIndex::variants() {
                            self.grid.get_from(x0, y0).get_from(x1, y1).number.set(None);
                        }
                    }
                }
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
    /// once - there is no way to be left with leftovers from the previous board.
    ///
    /// The whole thing is one transaction. Each write fans out to the cell's twenty peers
    /// through `possible_values`, and to the rest of its row, column and block again through
    /// `possible_values_last`; done one at a time, the grid would re-render after every
    /// character and spend most of its time on states no one asked to see.
    fn load(&self, board: &Board, name: &str) {
        log::info!("loading {name}");

        get_driver().transaction(|_| {
            // Walked through `variants()` rather than by arithmetic on an index, so the
            // mapping from a character to a cell cannot fail. The first index of `get_from`
            // selects the row and the second the column, at both levels - which is the order
            // `MainRender` lays the nine blocks out in.
            for (block_row_n, block_row) in TreeBoxIndex::variants().into_iter().enumerate() {
                for (in_row_n, in_row) in TreeBoxIndex::variants().into_iter().enumerate() {
                    let row = board[block_row_n * 3 + in_row_n].as_bytes();

                    for (block_col_n, block_col) in TreeBoxIndex::variants().into_iter().enumerate()
                    {
                        for (in_col_n, in_col) in TreeBoxIndex::variants().into_iter().enumerate() {
                            let value = row
                                .get(block_col_n * 3 + in_col_n)
                                .copied()
                                .and_then(SudokuValue::from_ascii);

                            self.grid
                                .get_from(block_row, block_col)
                                .get_from(in_row, in_col)
                                .number
                                .set(value);
                        }
                    }
                }
            }
        });
    }
}
