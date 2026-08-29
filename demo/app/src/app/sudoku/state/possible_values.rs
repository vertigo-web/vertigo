use std::collections::HashSet;
use vertigo::Computed;

use super::{
    number_item::{NumberItem, SudokuValue},
    sudoku_square::SudokuSquare,
    tree_box::TreeBoxIndex,
};

pub type PossibleValues = Computed<HashSet<SudokuValue>>;

pub fn possible_values(
    grid: &SudokuSquare<SudokuSquare<NumberItem>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex,
) -> Computed<HashSet<SudokuValue>> {
    let grid = grid.clone();
    Computed::from(move |context| {
        // Everything, less whatever is already taken in this cell's row, column and block.
        let mut current_numbers_in_ceis: HashSet<SudokuValue> =
            SudokuValue::variants().into_iter().collect();

        // Iterate by row
        for x0 in TreeBoxIndex::variants() {
            for x1 in TreeBoxIndex::variants() {
                let value = grid.get_from(x0, level0y).get_from(x1, level1y);
                let value = value.get(context);
                if let Some(value) = value {
                    current_numbers_in_ceis.remove(&value);
                }
            }
        }

        // Iterate by column
        for y0 in TreeBoxIndex::variants() {
            for y1 in TreeBoxIndex::variants() {
                let value = grid.get_from(level0x, y0).get_from(level1x, y1);
                let value = value.get(context);
                if let Some(value) = value {
                    current_numbers_in_ceis.remove(&value);
                }
            }
        }

        // Iterate by square
        for x1 in TreeBoxIndex::variants() {
            for y1 in TreeBoxIndex::variants() {
                let value = grid.get_from(level0x, level0y).get_from(x1, y1);
                let value = value.get(context);
                if let Some(value) = value {
                    current_numbers_in_ceis.remove(&value);
                }
            }
        }

        current_numbers_in_ceis
    })
}
