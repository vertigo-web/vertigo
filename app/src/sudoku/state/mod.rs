use virtualdom::computed::{
    Computed,
    Dependencies,
    Value
};

use self::{
    number_item::NumberItem,
    possible_values::{
        PossibleValues,
        possible_values
    },
    possible_values_last::{
        PossibleValuesLast,
        possible_values_last
    },
    sudoku_square::SudokuSquare,
    tree_box::TreeBoxIndex
};

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
                show_delete: deps.newValue(true)
            }
        });
    });
}

pub struct Sudoku {
    deps: Dependencies,
    pub grid: SudokuSquare<SudokuSquare<Cell>>,
}

impl Sudoku {
    pub fn new(deps: &Dependencies) -> Computed<Sudoku> {
        let gridNumber = createGrid(deps);
        let gridPossible = createGridPossible(deps, &gridNumber);
        let gridPossibleLast = createGridPossibleLast(deps, &gridNumber, &gridPossible);

        deps.newComputedFrom(Sudoku {
            deps: deps.clone(),
            grid: creatergidView(deps, gridNumber, gridPossible, gridPossibleLast),
        })
    }

    pub fn clear(&self) {
        log::info!("clear");

        self.deps.transaction(||{
            for x0 in TreeBoxIndex::variants() {
                for y0 in TreeBoxIndex::variants() {
                    for x1 in TreeBoxIndex::variants() {
                        for y1 in TreeBoxIndex::variants() {
                            self.grid.getFrom(x0, y0).getFrom(x1, y1).number.value.setValue(None);
                        }
                    }
                }
            }
        });
    }

    pub fn example1(&self) {
        log::info!("przykład1");
    }

    pub fn example2(&self) {
        log::info!("przykład2");
    }

    pub fn example3(&self) {
        log::info!("przykład3");
    }
}

