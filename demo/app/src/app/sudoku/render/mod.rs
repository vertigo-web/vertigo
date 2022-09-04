use vertigo::{css, css_fn, Css, bind, DomElement, dom};

use self::config::Config;
use super::state::{sudoku_square::SudokuSquare, tree_box::TreeBoxIndex, Cell, Sudoku};

pub mod config;
pub mod render_cell_possible;
pub mod render_cell_value;

css_fn! { css_center, "
    display: flex;
    justify-content: center;
" }

fn css_wrapper() -> Css {
    let config = Config::new();
    css!(
        "
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: {config.all_width}px;
        height: {config.all_width}px;

        border: 2px solid blue;
        user-select: none;
    "
    )
}

fn css_item_wrapper() -> Css {
    let config = Config::new();
    css!(
        "
        border: {config.group_border_size}px solid black;

        width: {config.group_width_size}px;
        height: {config.group_width_size}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;
    "
    )
}

fn css_cell_wrapper() -> Css {
    let config = Config::new();
    css!(
        "
        border: {config.item_border_size}px solid green;
        width: {config.item_width_size}px;
        height: {config.item_width_size}px;
    "
    )
}

fn render_cell(item: &Cell) -> DomElement {
    let item = item.clone();

    let value_view = item.number.value.render_value({
        let item = item.clone();
        move |value| {
            if let Some(value) = value {
                render_cell_value::render_cell_value(value, &item)
            } else {
                render_cell_possible::render_cell_possible(&item)
            }
        }
    });

    dom! {
        <div css={css_cell_wrapper()}>
            { value_view }
        </div>
    }
}

fn render_group(group: &SudokuSquare<Cell>) -> DomElement {
    let view1 = render_cell(group.get_from(TreeBoxIndex::First , TreeBoxIndex::First ));
    let view2 = render_cell(group.get_from(TreeBoxIndex::First , TreeBoxIndex::Middle));
    let view3 = render_cell(group.get_from(TreeBoxIndex::First , TreeBoxIndex::Last  ));
    let view4 = render_cell(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First ));
    let view5 = render_cell(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle));
    let view6 = render_cell(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last  ));
    let view7 = render_cell(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::First ));
    let view8 = render_cell(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Middle));
    let view9 = render_cell(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Last  ));

    dom! {
        <div css={css_item_wrapper()}>
            { view1 }
            { view2 }
            { view3 }
            { view4 }
            { view5 }
            { view6 }
            { view7 }
            { view8 }
            { view9 }
        </div>
    }
}

pub fn main_render(sudoku: &Sudoku) -> DomElement {
    let view1 = examples_render(sudoku);
    let view2 = main_render_inner(sudoku);

    let wrapper_css = css!{"
        display: flex;
    "};

    dom! {
        <div css={wrapper_css}>
            { view1 }
            { view2 }
        </div>
    }
}

pub fn main_render_inner(sudoku: &Sudoku) -> DomElement {
    let view1 = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::First ));
    let view2 = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::Middle));
    let view3 = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::Last  ));
    let view4 = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First ));
    let view5 = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle));
    let view6 = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last  ));
    let view7 = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::First ));
    let view8 = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Middle));
    let view9 = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Last  ));

    dom! {
        <div css={css_center()}>
            <div css={css_wrapper()}>
                { view1 }
                { view2 }
                { view3 }
                { view4 }
                { view5 }
                { view6 }
                { view7 }
                { view8 }
                { view9 }
            </div>
        </div>
    }
}

css_fn! { css_sudoku_example, "
    border: 1px solid black;
    padding: 10px;
    flex-shrink: 1;
    display: flex;
    flex-direction: column;
    margin-right: 10px;
" }

css_fn! { css_sudoku_example_button, "
    margin: 5px;
    cursor: pointer;
" }

pub fn examples_render(sudoku: &Sudoku) -> DomElement {
    let clear = bind(sudoku).call(|_, sudoku| {
        sudoku.clear();
    });

    let example1 = bind(sudoku).call(|_, sudoku| {
        sudoku.example1();
    });

    let example2 = bind(sudoku).call(|_, sudoku| {
        sudoku.example2();
    });

    let example3 = bind(sudoku).call(|_, sudoku| {
        sudoku.example3();
    });

    dom! {
        <div css={css_sudoku_example()}>
            <button css={css_sudoku_example_button()} on_click={clear}>"Clear"</button>
            <button css={css_sudoku_example_button()} on_click={example1}>"Example 1"</button>
            <button css={css_sudoku_example_button()} on_click={example2}>"Example 2"</button>
            <button css={css_sudoku_example_button()} on_click={example3}>"Example 3"</button>
        </div>
    }
}
