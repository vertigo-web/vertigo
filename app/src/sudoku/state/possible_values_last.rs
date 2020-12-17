use std::collections::HashSet;

use virtualdom::computed::{
    Computed,
    Dependencies
};

use super::{number_item::{NumberItem, SudokuValue}, possible_values::PossibleValues, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

fn iterateBy<
    F: Fn(TreeBoxIndex, TreeBoxIndex) -> bool
>(
    isNumberFilled: &F
) -> Vec<(TreeBoxIndex, TreeBoxIndex)> {
    let mut out: Vec<(TreeBoxIndex, TreeBoxIndex)> = Vec::new();

    for x0 in TreeBoxIndex::variants() {
        for x1 in TreeBoxIndex::variants() {
            if isNumberFilled(x0, x1) == false {
                out.push((x0, x1));
            }
        }
    }

    out
}

fn getPossibleValue<
    F: Fn(TreeBoxIndex, TreeBoxIndex) -> bool,
    S: Fn(TreeBoxIndex, TreeBoxIndex) -> Computed<HashSet<SudokuValue>>,
>(
    isNumberFilled: F,
    current: Computed<HashSet<SudokuValue>>,
    selectFromGrid: S
) -> Option<SudokuValue> {
    for possibleValue in (*current.getValue()).iter() {

        let mut count = 0;

        for (check_x0, check_x1) in iterateBy(&isNumberFilled) {
            if selectFromGrid(check_x0, check_x1).getValue().contains(&possibleValue) {
                count += 1;
            }
        }

        if count == 1 {
            return Some(*possibleValue);
        }
    }

    None
}

pub type PossibleValuesLast = Computed<Option<SudokuValue>>;

fn valueByRow(
    deps: &Dependencies,
    gridInput: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid: &SudokuSquare<SudokuSquare<PossibleValues>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let gridInput = (*gridInput).clone();
    let grid = (*grid).clone();

    deps.from(move || {
        let isNumberFilled = {
            let gridInput = gridInput.clone();
            move |x0, x1| -> bool {
                gridInput.getFrom(x0, level0y).getFrom(x1, level1y).value.getValue().is_some()
            }
        };

        let getCurrent = (*grid.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

        let selectFromGrid = {
            let grid = grid.clone();
            move |x0, x1| -> Computed<HashSet<SudokuValue>> {
                (*grid.getFrom(x0, level0y).getFrom(x1, level1y)).clone()
            }
        };

        //iterowaine po wierszu
        getPossibleValue(isNumberFilled, getCurrent, selectFromGrid)
    })
}


fn valueByCol(
    deps: &Dependencies,
    gridInput: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid: &SudokuSquare<SudokuSquare<PossibleValues>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let gridInput = (*gridInput).clone();
    let grid = (*grid).clone();

    deps.from(move || {
        let isNumberFilled = {
            let gridInput = gridInput.clone();
            move |y0, y1| -> bool {
                gridInput.getFrom(level0x, y0).getFrom(level1x, y1).value.getValue().is_some()
            }
        };

        let getCurrent = (*grid.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

        let selectFromGrid = {
            let grid = grid.clone();
            move |y0, y1| -> Computed<HashSet<SudokuValue>> {
                (*grid.getFrom(level0x, y0).getFrom(level1x, y1)).clone()
            }
        };

        //iterowanie po kolumnie
        getPossibleValue(isNumberFilled, getCurrent, selectFromGrid)
    })
}

fn valueBySquare(
    deps: &Dependencies,
    gridInput: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid: &SudokuSquare<SudokuSquare<PossibleValues>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let gridInput = (*gridInput).clone();
    let grid = (*grid).clone();

    deps.from(move || {
        let isNumberFilled = {
            let gridInput = gridInput.clone();
            move |x1, y1| -> bool {
                gridInput.getFrom(level0x, level0y).getFrom(x1, y1).value.getValue().is_some()
            }
        };

        let getCurrent = (*grid.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

        let selectFromGrid = {
            let grid = grid.clone();
            move |x1, y1| -> Computed<HashSet<SudokuValue>> {
                (*grid.getFrom(level0x, level0y).getFrom(x1, y1)).clone()
            }
        };

        //iterowanie po kwadracie
        getPossibleValue(isNumberFilled, getCurrent, selectFromGrid)
    })
}

pub fn possible_values_last(
    deps: &Dependencies,
    gridInput: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid: &SudokuSquare<SudokuSquare<PossibleValues>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let byRow = valueByRow(deps, gridInput, grid, level0x, level0y, level1x, level1y);
    let byCol = valueByCol(deps, gridInput, grid, level0x, level0y, level1x, level1y);
    let bySquare = valueBySquare(deps, gridInput, grid, level0x, level0y, level1x, level1y);
    
    deps.from(move || {
        let by_row = *byRow.getValue();
        if let Some(by_row) = by_row {
            return Some(by_row);
        }

        let by_col = *byCol.getValue();
        if let Some(by_col) = by_col {
            return Some(by_col);
        }

        let by_square = *bySquare.getValue();
        if let Some(by_square) = by_square {
            return Some(by_square);
        }

        None
    })
}
