use std::cmp::PartialEq;
use vertigo::computed::{Computed, Dependencies, Value};

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


fn create_grid(deps: &Dependencies,) -> SudokuSquare<SudokuSquare<NumberItem>> {
    SudokuSquare::create_with_iterator(move |level0x, level0y| {
        SudokuSquare::create_with_iterator(move |level1x, level1y| {
            NumberItem::new(deps, level0x, level0y, level1x, level1y, None)
        })
    })
}

fn create_grid_possible(deps: &Dependencies, grid_number: &SudokuSquare<SudokuSquare<NumberItem>>) -> SudokuSquare<SudokuSquare<PossibleValues>> {
    SudokuSquare::create_with_iterator(|level0x, level0y| {
        SudokuSquare::create_with_iterator(|level1x, level1y| {
            possible_values(deps, grid_number, level0x, level0y, level1x, level1y)
        })
    })
}

fn create_grid_possible_last(
    deps: &Dependencies,
    grid_number: &SudokuSquare<SudokuSquare<NumberItem>>,
    grid_possible: &SudokuSquare<SudokuSquare<PossibleValues>>
) -> SudokuSquare<SudokuSquare<PossibleValuesLast>> {
    SudokuSquare::create_with_iterator(|level0x, level0y| {
        SudokuSquare::create_with_iterator(|level1x, level1y| {
            possible_values_last(deps, grid_number, grid_possible, level0x, level0y, level1x, level1y)
        })
    })
}

#[derive(PartialEq, Clone)]
pub struct Cell {
    pub number: NumberItem,
    pub possible: PossibleValues,
    pub possible_last: PossibleValuesLast,
    pub show_delete: Value<bool>,
}

fn creatergid_view(
    deps: &Dependencies,
    grid_number: SudokuSquare<SudokuSquare<NumberItem>>,
    grid_possible: SudokuSquare<SudokuSquare<PossibleValues>>,
    grid_possible_last: SudokuSquare<SudokuSquare<PossibleValuesLast>>,
) -> SudokuSquare<SudokuSquare<Cell>> {

    SudokuSquare::create_with_iterator(|level0x, level0y| {
        SudokuSquare::create_with_iterator(|level1x, level1y| {
            let number = grid_number.get_from(level0x, level0y).get_from(level1x, level1y);
            let possible = grid_possible.get_from(level0x, level0y).get_from(level1x, level1y);
            let possible_last = grid_possible_last.get_from(level0x, level0y).get_from(level1x, level1y);

            Cell {
                number,
                possible,
                possible_last,
                show_delete: deps.new_value(true)
            }
        })
    })
}

#[derive(PartialEq)]
pub struct Sudoku {
    deps: Dependencies,
    pub grid: SudokuSquare<SudokuSquare<Cell>>,
}

impl Sudoku {
    pub fn new(deps: &Dependencies) -> Computed<Sudoku> {
        let grid_number = create_grid(deps);
        let grid_possible = create_grid_possible(deps, &grid_number);
        let grid_possible_last = create_grid_possible_last(deps, &grid_number, &grid_possible);

        deps.new_computed_from(Sudoku {
            deps: deps.clone(),
            grid: creatergid_view(deps, grid_number, grid_possible, grid_possible_last),
        })
    }

    pub fn clear(&self) {
        log::info!("clear");

        self.deps.transaction(|| {
            for x0 in TreeBoxIndex::variants() {
                for y0 in TreeBoxIndex::variants() {
                    for x1 in TreeBoxIndex::variants() {
                        for y1 in TreeBoxIndex::variants() {
                            self.grid.get_from(x0, y0).get_from(x1, y1).number.value.set_value(None);
                        }
                    }
                }
            }
        });
    }

    pub fn example1(&self) {
        log::info!("example 1");
    }

    pub fn example2(&self) {
        log::info!("example 2");
    }

    pub fn example3(&self) {
        log::info!("example 3");
    }
}
