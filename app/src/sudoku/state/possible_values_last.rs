use virtualdom::computed::{
    Computed,
    Dependencies
};

use super::{number_item::{NumberItem, SudokuValue}, possible_values::PossibleValues, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

fn iterateBy() -> Vec<(TreeBoxIndex, TreeBoxIndex)> {
    let mut out: Vec<(TreeBoxIndex, TreeBoxIndex)> = Vec::new();

    for x0 in TreeBoxIndex::variants() {
        for x1 in TreeBoxIndex::variants() {
            out.push((x0, x1));
        }
    }

    out
}

#[derive(Clone)]
struct CellForComputed {
    pub input: NumberItem,
    pub possible: PossibleValues,
}

fn getPossibleValue<
    S: Fn(TreeBoxIndex, TreeBoxIndex) -> CellForComputed,
>(
    current: CellForComputed,
    selectFromGrid: S
) -> Option<SudokuValue> {
    for possibleValue in (*current.possible.getValue()).iter() {

        let mut count = 0;

        for (check_x0, check_x1) in iterateBy() {
            let cell = selectFromGrid(check_x0, check_x1);

            let input_value = cell.input.value.getValue();

            if input_value.is_none() && cell.possible.getValue().contains(&possibleValue) {
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
    gridComputed: &SudokuSquare<SudokuSquare<CellForComputed>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let gridComputed = (*gridComputed).clone();

    deps.from(move || {
        let getCurrent = (*gridComputed.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

        //iterowaine po wierszu
        getPossibleValue(getCurrent, {
            let grid = gridComputed.clone();
            move |x0, x1| -> CellForComputed {
                (*grid.getFrom(x0, level0y).getFrom(x1, level1y)).clone()
            }
        })
    })
}


fn valueByCol(
    deps: &Dependencies,
    gridComputed: &SudokuSquare<SudokuSquare<CellForComputed>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let gridComputed = (*gridComputed).clone();

    deps.from(move || {
        let getCurrent = (*gridComputed.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

        //iterowanie po kolumnie
        getPossibleValue(getCurrent, {
            let grid = gridComputed.clone();
            move |y0, y1| -> CellForComputed {
                (*grid.getFrom(level0x, y0).getFrom(level1x, y1)).clone()
            }
        })
    })
}

fn valueBySquare(
    deps: &Dependencies,
    gridComputed: &SudokuSquare<SudokuSquare<CellForComputed>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {
    let gridComputed = (*gridComputed).clone();
    deps.from(move || {

        let getCurrent = (*gridComputed.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

        //iterowanie po kwadracie
        getPossibleValue(getCurrent, {
            let grid = gridComputed.clone();
            move |x1, y1| -> CellForComputed {
                (*grid.getFrom(level0x, level0y).getFrom(x1, y1)).clone()
            }
        })
    })
}

pub fn possible_values_last(
    deps: &Dependencies,
    gridInput: &SudokuSquare<SudokuSquare<NumberItem>>,
    gridPossible: &SudokuSquare<SudokuSquare<PossibleValues>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex
) -> Computed<Option<SudokuValue>> {

    let gridComputed: SudokuSquare<SudokuSquare<CellForComputed>> = {
        SudokuSquare::createWithIterator(|level0x, level0y| {
            SudokuSquare::createWithIterator(|level1x, level1y| {
                let input = (*gridInput.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();
                let possible = (*gridPossible.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();
    
                CellForComputed {
                    input,
                    possible,
                }
            })
        })
    };

    let byRow = valueByRow(deps, &gridComputed, level0x, level0y, level1x, level1y);
    let byCol = valueByCol(deps, &gridComputed, level0x, level0y, level1x, level1y);
    let bySquare = valueBySquare(deps, &gridComputed, level0x, level0y, level1x, level1y);
    
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
