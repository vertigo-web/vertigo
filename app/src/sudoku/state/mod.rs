use virtualdom::computed::{Computed::Computed, Dependencies::Dependencies, Value::Value};

use self::{number_item::{NumberItem, SudokuValue}, possible_values::{PossibleValues, possible_values}, possible_values_last::{PossibleValuesLast, possible_values_last}, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

pub mod tree_box;
pub mod sudoku_square;
pub mod number_item;
pub mod possible_values;
pub mod possible_values_last;


fn createGrid(deps: &Dependencies,) -> SudokuSquare<SudokuSquare<NumberItem>> {
    SudokuSquare::createWithIterator(move |level0x, level0y| {
        SudokuSquare::createWithIterator(move |level1x, level1y| {
            NumberItem::new(deps, level0x, level0y, level1x, level1y, None)
        })
    })
}

fn createGridPossible(deps: &Dependencies, gridNumber: &SudokuSquare<SudokuSquare<NumberItem>>) -> SudokuSquare<SudokuSquare<PossibleValues>> {
    SudokuSquare::createWithIterator(|level0x, level0y| {
        SudokuSquare::createWithIterator(|level1x, level1y| {
            possible_values(deps, gridNumber, level0x, level0y, level1x, level1y)
        })
    })
}

fn createGridPossibleLast(
    deps: &Dependencies,
    gridNumber: &SudokuSquare<SudokuSquare<NumberItem>>,
    gridPossible: &SudokuSquare<SudokuSquare<PossibleValues>>
) -> SudokuSquare<SudokuSquare<PossibleValuesLast>> {
    SudokuSquare::createWithIterator(|level0x, level0y| {
        SudokuSquare::createWithIterator(|level1x, level1y| {
            possible_values_last(deps, gridNumber, gridPossible, level0x, level0y, level1x, level1y)
        })
    })
}
pub struct Cell {
    pub number: NumberItem,
    pub possible: PossibleValues,
    pub possibleLast: PossibleValuesLast,
    pub show_delete: Value<bool>,
}

fn creatergidView(
    deps: &Dependencies,
    gridNumber: SudokuSquare<SudokuSquare<NumberItem>>,
    gridPossible: SudokuSquare<SudokuSquare<PossibleValues>>,
    gridPossibleLast: SudokuSquare<SudokuSquare<PossibleValuesLast>>,
) -> SudokuSquare<SudokuSquare<Cell>> {

    return SudokuSquare::createWithIterator(|level0x, level0y| {
        return SudokuSquare::createWithIterator(|level1x, level1y| {
            let number = (*gridNumber.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();
            let possible = (*gridPossible.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();
            let possibleLast = (*gridPossibleLast.getFrom(level0x, level0y).getFrom(level1x, level1y)).clone();

            Cell {
                number,
                possible,
                possibleLast,
                show_delete: deps.newValue(false)
            }
        });
    });
}

pub struct Sudoku {
    pub grid: SudokuSquare<SudokuSquare<Cell>>,
}

impl Sudoku {
    pub fn new(deps: &Dependencies) -> Computed<Sudoku> {
        let gridNumber = createGrid(deps);
        gridNumber.getFrom(TreeBoxIndex::First, TreeBoxIndex::First).getFrom(TreeBoxIndex::First, TreeBoxIndex::First).value.setValue(Some(SudokuValue::Value2));         //TODO - testowo
        let gridPossible = createGridPossible(deps, &gridNumber);
        let gridPossibleLast = createGridPossibleLast(deps, &gridNumber, &gridPossible);

        deps.newComputedFrom(Sudoku {
            grid: creatergidView(deps, gridNumber, gridPossible, gridPossibleLast),
        })
    }
}

