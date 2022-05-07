use vertigo::{ Value, VDomComponent, get_driver};

use self::{
    number_item::NumberItem,
    possible_values::{possible_values, PossibleValues},
    possible_values_last::{possible_values_last, PossibleValuesLast},
    sudoku_square::SudokuSquare,
    tree_box::TreeBoxIndex,
};

pub mod number_item;
pub mod possible_values;
pub mod possible_values_last;
pub mod sudoku_square;
pub mod tree_box;

fn create_grid() -> SudokuSquare<SudokuSquare<NumberItem>> {
    SudokuSquare::create_with_iterator(move |level0x, level0y| {
        SudokuSquare::create_with_iterator(move |level1x, level1y| {
            NumberItem::new(level0x, level0y, level1x, level1y, None)
        })
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
            possible_values_last(grid_number, grid_possible, level0x, level0y, level1x, level1y)
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

fn creatergid_view(
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
                number: number.clone(),
                possible: possible.clone(),
                possible_last: possible_last.clone(),
                show_delete: Value::new(true),
            }
        })
    })
}

#[derive(Clone)]
pub struct Sudoku {
    pub grid: SudokuSquare<SudokuSquare<Cell>>,
}

impl Sudoku {
    pub fn component() -> VDomComponent {
        let grid_number = create_grid();
        let grid_possible = create_grid_possible(&grid_number);
        let grid_possible_last = create_grid_possible_last(&grid_number, &grid_possible);
            
        let state = Sudoku {
            grid: creatergid_view(grid_number, grid_possible, grid_possible_last),
        };

        crate::app::sudoku::render::main_render(state)
    }

    pub fn clear(&self) {
        log::info!("clear");

        get_driver().transaction(|| {
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
