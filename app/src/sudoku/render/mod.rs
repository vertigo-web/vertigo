use alloc::{
    vec,
    format,
};
use vertigo::{computed::Computed, VDomNode, node_attr, Css};
use self::config::Config;

use super::state::{Cell, Sudoku, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

pub mod config;
pub mod render_cell_value;
pub mod render_cell_possible;

fn css_center() -> Css {
    Css::one("
        display: flex;
        justify-content: center;
    ")
}

fn css_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: {}px;
        height: {}px;

        border: 2px solid blue;
    ", config.all_width, config.all_width))
}

fn css_item_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        border: {}px solid black;

        width: {}px;
        height: {}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;
    ", config.group_border_size, config.group_width_size, config.group_width_size))
}

fn css_cell_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        border: {}px solid green;
        width: {}px;
        height: {}px;
    ", config.item_border_size, config.item_width_size, config.item_width_size))
}

fn render_cell(item: &Computed<Cell>) -> VDomNode {
    let value = *item.get_value().number.value.get_value();

    // log::warn!("cell {:?}", value);
    if let Some(value) = value {
        return render_cell_value::render_cell_value(value, item);
    }

    render_cell_possible::render_cell_possible(item)
}

fn render_group(group: &Computed<SudokuSquare<Cell>>) -> VDomNode {
    use node_attr::{buildNode, node, css, component};

    //log::info!("render group");

    let get_cell = |group: &Computed<SudokuSquare<Cell>>, x: TreeBoxIndex, y: TreeBoxIndex| -> Computed<Cell> {
        group.clone().map(move |state| {
            state.get_value().get_from(x, y)
        })
    };

    buildNode("div", vec!(
        css(css_item_wrapper()),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::First,  TreeBoxIndex::First),  render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::First,  TreeBoxIndex::Middle), render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::First,  TreeBoxIndex::Last),   render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::Middle, TreeBoxIndex::First),  render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::Middle, TreeBoxIndex::Middle), render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::Middle, TreeBoxIndex::Last),   render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::Last,   TreeBoxIndex::First),  render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::Last,   TreeBoxIndex::Middle), render_cell),
        )),
        node("div", vec!(
            css(css_cell_wrapper()),
            component(get_cell(group, TreeBoxIndex::Last,   TreeBoxIndex::Last),   render_cell),
        ))
    ))
}

pub fn main_render(sudoku: &Computed<Sudoku>) -> VDomNode {
    use node_attr::{buildNode, node, css, component};

    let get_group = |sudoku: &Computed<Sudoku>, x: TreeBoxIndex, y: TreeBoxIndex| -> Computed<SudokuSquare<Cell>> {
        sudoku.clone().map(move |state| {
            state.get_value().grid.get_from(x, y)
        })
    };
    
    buildNode("div", vec!(
        css(css_center()),
        node("div", vec!(
            css(css_wrapper()),
            component(get_group(sudoku, TreeBoxIndex::First,  TreeBoxIndex::First),  render_group),
            component(get_group(sudoku, TreeBoxIndex::First,  TreeBoxIndex::Middle), render_group),
            component(get_group(sudoku, TreeBoxIndex::First,  TreeBoxIndex::Last),   render_group),
            component(get_group(sudoku, TreeBoxIndex::Middle, TreeBoxIndex::First),  render_group),
            component(get_group(sudoku, TreeBoxIndex::Middle, TreeBoxIndex::Middle), render_group),
            component(get_group(sudoku, TreeBoxIndex::Middle, TreeBoxIndex::Last),   render_group),
            component(get_group(sudoku, TreeBoxIndex::Last,   TreeBoxIndex::First),  render_group),
            component(get_group(sudoku, TreeBoxIndex::Last,   TreeBoxIndex::Middle), render_group),
            component(get_group(sudoku, TreeBoxIndex::Last,   TreeBoxIndex::Last),   render_group),
        ))
    ))
}

fn css_sudoku_example() -> Css {
    Css::one("
        border: 1px solid black;
        padding: 10px;
        margin: 10px 0;
    ")
}

fn css_sudoku_example_button() -> Css {
    Css::one("
        margin: 5px;
        cursor: pointer;
    ")
}
pub fn examples_render(sudoku: &Computed<Sudoku>) -> VDomNode {
    use node_attr::{buildNode, node, css, text, onClick};

    let sudoku = sudoku.get_value();
    buildNode("div", vec!(
        css(css_sudoku_example()),
        node("button", vec!(
            css(css_sudoku_example_button()),
            onClick({
                let sudoku = sudoku.clone();

                move || {
                    sudoku.clear();
                }
            }),
            text("Wyczyść")
        )),
        node("button", vec!(
            css(css_sudoku_example_button()),
            onClick({
                let sudoku = sudoku.clone();

                move || {
                    sudoku.example1();
                }
            }),
            text("Przykład 1")
        )),
        node("button", vec!(
            css(css_sudoku_example_button()),
            onClick({
                let sudoku = sudoku.clone();

                move || {
                    sudoku.example2();
                }
            }),
            text("Przykład 2")
        )),
        node("button", vec!(
            css(css_sudoku_example_button()),
            onClick({
                let sudoku = sudoku.clone();

                move || {
                    sudoku.example3();
                }
            }),
            text("Przykład 3")
        ))
    ))
}
