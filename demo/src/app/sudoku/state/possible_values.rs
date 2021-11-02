use vertigo::computed::{
    Computed,
    Dependencies
};

use super::{
    number_item::{
        NumberItem,
        SudokuValue
    },
    sudoku_square::SudokuSquare, tree_box::TreeBoxIndex
};

use std::collections::HashSet;

pub type PossibleValues = Computed<HashSet<SudokuValue>>;

pub fn possible_values(
    deps: &Dependencies,
    grid: &SudokuSquare<SudokuSquare<NumberItem>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex,
) -> Computed<HashSet<SudokuValue>> {
    let grid = grid.clone();
    deps.from(move || {
        let mut current_numbers_in_ceis: HashSet<SudokuValue> = HashSet::new();
        current_numbers_in_ceis.insert(SudokuValue::Value1);
        current_numbers_in_ceis.insert(SudokuValue::Value2);
        current_numbers_in_ceis.insert(SudokuValue::Value3);
        current_numbers_in_ceis.insert(SudokuValue::Value4);
        current_numbers_in_ceis.insert(SudokuValue::Value5);
        current_numbers_in_ceis.insert(SudokuValue::Value6);
        current_numbers_in_ceis.insert(SudokuValue::Value7);
        current_numbers_in_ceis.insert(SudokuValue::Value8);
        current_numbers_in_ceis.insert(SudokuValue::Value9);

        // Iterate by row
        for x0 in TreeBoxIndex::variants() {
            for x1 in TreeBoxIndex::variants() {
                let value = grid.get_from(x0, level0y).get_from(x1, level1y);
                let value = *value.value.get_value();
                if let Some(value) = value {
                    current_numbers_in_ceis.remove(&value);
                }
            }
        }

        // Iterate by column
        for y0 in TreeBoxIndex::variants() {
            for y1 in TreeBoxIndex::variants() {
                let value = grid.get_from(level0x, y0).get_from(level1x, y1);
                let value = *value.value.get_value();
                if let Some(value) = value {
                    current_numbers_in_ceis.remove(&value);
                }
            }
        }

        // Iterate by square
        for x1 in TreeBoxIndex::variants() {
            for y1 in TreeBoxIndex::variants() {
                let value = grid.get_from(level0x, level0y).get_from(x1, y1);
                let value = *value.value.get_value();
                if let Some(value) = value {
                    current_numbers_in_ceis.remove(&value);
                }
            }
        }

        current_numbers_in_ceis
    })
}
