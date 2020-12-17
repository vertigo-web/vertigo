use virtualdom::computed::{
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
        let mut currentNumbersInCeis: HashSet<SudokuValue> = HashSet::new();
        currentNumbersInCeis.insert(SudokuValue::Value1);
        currentNumbersInCeis.insert(SudokuValue::Value2);
        currentNumbersInCeis.insert(SudokuValue::Value3);
        currentNumbersInCeis.insert(SudokuValue::Value4);
        currentNumbersInCeis.insert(SudokuValue::Value5);
        currentNumbersInCeis.insert(SudokuValue::Value6);
        currentNumbersInCeis.insert(SudokuValue::Value7);
        currentNumbersInCeis.insert(SudokuValue::Value8);
        currentNumbersInCeis.insert(SudokuValue::Value9);

        //iterowaine po wierszu
        for x0 in TreeBoxIndex::variants() {
            for x1 in TreeBoxIndex::variants() {
                let value = grid.getFrom(x0, level0y).getFrom(x1, level1y);
                let value = *value.value.getValue();
                if let Some(value) = value {
                    currentNumbersInCeis.remove(&value);
                }
            }
        }

        //iterowanie po kolumnie
        for y0 in TreeBoxIndex::variants() {
            for y1 in TreeBoxIndex::variants() {
                let value = grid.getFrom(level0x, y0).getFrom(level1x, y1);
                let value = *value.value.getValue();
                if let Some(value) = value {
                    currentNumbersInCeis.remove(&value);
                }
            }
        }

        //iterowanie po kwadracie
        for x1 in TreeBoxIndex::variants() {
            for y1 in TreeBoxIndex::variants() {
                let value = grid.getFrom(level0x, level0y).getFrom(x1, y1);
                let value = *value.value.getValue();
                if let Some(value) = value {
                    currentNumbersInCeis.remove(&value);
                }
            }
        }

        currentNumbersInCeis
    })
}
