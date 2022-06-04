use vertigo::{Context, Computed};

use super::{
    number_item::{NumberItem, SudokuValue},
    possible_values::PossibleValues,
    sudoku_square::SudokuSquare,
    tree_box::TreeBoxIndex,
};

fn iterate_by() -> Vec<(TreeBoxIndex, TreeBoxIndex)> {
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

fn get_possible_value<S>(context: &Context, current: &CellForComputed, select_from_grid: S) -> Option<SudokuValue>
where
    S: Fn(TreeBoxIndex, TreeBoxIndex) -> CellForComputed,
{
    for possible_value in current.possible.get(context).iter() {
        let mut count = 0;

        for (check_x0, check_x1) in iterate_by() {
            let cell = select_from_grid(check_x0, check_x1);

            let input_value = cell.input.value.get(context);

            if input_value.is_none() && cell.possible.get(context).contains(possible_value) {
                count += 1;
            }
        }

        if count == 1 {
            return Some(*possible_value);
        }
    }

    None
}

pub type PossibleValuesLast = Computed<Option<SudokuValue>>;

fn value_by_row(
    grid_computed: &SudokuSquare<SudokuSquare<CellForComputed>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex,
) -> Computed<Option<SudokuValue>> {
    let grid_computed = (*grid_computed).clone();

    Computed::from(move |context| {
        let get_current = grid_computed.get_from(level0x, level0y).get_from(level1x, level1y);

        // Iterate by row
        get_possible_value(context, get_current, {
            let grid = grid_computed.clone();
            move |x0, x1| -> CellForComputed { grid.get_from(x0, level0y).get_from(x1, level1y).clone() }
        })
    })
}

fn value_by_col(
    grid_computed: &SudokuSquare<SudokuSquare<CellForComputed>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex,
) -> Computed<Option<SudokuValue>> {
    let grid_computed = (*grid_computed).clone();

    Computed::from(move |context| {
        let get_current = grid_computed.get_from(level0x, level0y).get_from(level1x, level1y);

        // Iterate by column
        get_possible_value(context, get_current, {
            let grid = grid_computed.clone();
            move |y0, y1| -> CellForComputed { grid.get_from(level0x, y0).get_from(level1x, y1).clone() }
        })
    })
}

fn value_by_square(
    grid_computed: &SudokuSquare<SudokuSquare<CellForComputed>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex,
) -> Computed<Option<SudokuValue>> {
    let grid_computed = (*grid_computed).clone();
    Computed::from(move |context| {
        let get_current = grid_computed.get_from(level0x, level0y).get_from(level1x, level1y);

        // Iterate by square
        get_possible_value(context, get_current, {
            let grid = grid_computed.clone();
            move |x1, y1| -> CellForComputed { grid.get_from(level0x, level0y).get_from(x1, y1).clone() }
        })
    })
}

pub fn possible_values_last(
    grid_input: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid_possible: &SudokuSquare<SudokuSquare<PossibleValues>>,
    level0x: TreeBoxIndex,
    level0y: TreeBoxIndex,
    level1x: TreeBoxIndex,
    level1y: TreeBoxIndex,
) -> Computed<Option<SudokuValue>> {
    let grid_computed: SudokuSquare<SudokuSquare<CellForComputed>> = {
        SudokuSquare::create_with_iterator(|level0x, level0y| {
            SudokuSquare::create_with_iterator(|level1x, level1y| {
                let input = grid_input.get_from(level0x, level0y).get_from(level1x, level1y);
                let possible = grid_possible.get_from(level0x, level0y).get_from(level1x, level1y);

                CellForComputed { input: input.clone(), possible: possible.clone() }
            })
        })
    };

    let by_row = value_by_row(&grid_computed, level0x, level0y, level1x, level1y);
    let by_col = value_by_col(&grid_computed, level0x, level0y, level1x, level1y);
    let by_square = value_by_square(&grid_computed, level0x, level0y, level1x, level1y);

    Computed::from(move |context| {
        let by_row = by_row.get(context);
        if let Some(by_row) = by_row {
            return Some(by_row);
        }

        let by_col = by_col.get(context);
        if let Some(by_col) = by_col {
            return Some(by_col);
        }

        let by_square = by_square.get(context);
        if let Some(by_square) = by_square {
            return Some(by_square);
        }

        None
    })
}
