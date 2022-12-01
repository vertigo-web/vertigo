use vertigo::{css, bind, DomElement, dom};

use super::state::{sudoku_square::SudokuSquare, tree_box::TreeBoxIndex, Cell, SudokuState};

pub mod render_cell_possible;
pub mod render_cell_value;

pub struct Sudoku {
    pub state: SudokuState,
}

impl Sudoku {
    pub fn mount(&self) -> DomElement {
        let view1 = examples_render(&self.state);
        let view2 = main_render(&self.state);

        let wrapper_css = css!("
            display: flex;
        ");

        dom! {
            <div css={wrapper_css}>
                { view1 }
                { view2 }
            </div>
        }
    }
}

pub fn main_render(sudoku: &SudokuState) -> DomElement {
    let (group_width, group_height, view1) = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::First ));
    let (_, _, view2) = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::Middle));
    let (_, _, view3) = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::Last  ));
    let (_, _, view4) = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First ));
    let (_, _, view5) = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle));
    let (_, _, view6) = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last  ));
    let (_, _, view7) = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::First ));
    let (_, _, view8) = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Middle));
    let (_, _, view9) = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Last  ));

    let width = 3 * group_width + 2 * 2;
    let height = 3 * group_height + 2 * 2;

    let out_css = css!(
        "
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: {width}px;
        height: {height}px;

        border: 2px solid blue;
        user-select: none;
    "
    );


    let css_center = css!("
        display: flex;
        justify-content: center;
    ");

    dom! {
        <div css={css_center}>
            <div css={out_css}>
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

fn render_group(group: &SudokuSquare<Cell>) -> (u32, u32, DomElement) {
    let (cell_width, cell_height, view1) = render_cell(group.get_from(TreeBoxIndex::First , TreeBoxIndex::First ));
    let (_, _, view2) = render_cell(group.get_from(TreeBoxIndex::First , TreeBoxIndex::Middle));
    let (_, _, view3) = render_cell(group.get_from(TreeBoxIndex::First , TreeBoxIndex::Last  ));
    let (_, _, view4) = render_cell(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First ));
    let (_, _, view5) = render_cell(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle));
    let (_, _, view6) = render_cell(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last  ));
    let (_, _, view7) = render_cell(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::First ));
    let (_, _, view8) = render_cell(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Middle));
    let (_, _, view9) = render_cell(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Last  ));

    let border = 1;

    let width = 2 * border + 3 * cell_width;
    let height = 2 * border + 3 * cell_height;

    let out_css = css!(
        "
        border: {border}px solid black;

        width: {width}px;
        height: {height}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;
    "
    );

    let group = dom! {
        <div css={out_css}>
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
    };

    (width, height, group)
}

fn render_cell(item: &Cell) -> (u32, u32, DomElement) {
    let item = item.clone();

    let small_item_width = 24;
    let small_item_height = 24;
    let border = 1;

    let cell_width = 2 * border + 3 * small_item_width;
    let cell_height = 2 * border + 3 * small_item_height;

    let value_view = item.number.value.render_value({
        let item = item.clone();
        move |value| {
            if let Some(value) = value {
                render_cell_value::render_cell_value(cell_height, value, &item)
            } else {
                render_cell_possible::render_cell_possible(cell_width, &item)
            }
        }
    });

    let css_wrapper = css!(
        "
        border: {border}px solid green;
        width: {cell_width}px;
        height: {cell_height}px;
    "
    );

    let dom = dom! {
        <div css={css_wrapper}>
            { value_view }
        </div>
    };

    (cell_width, cell_height, dom)
}

fn examples_render(sudoku: &SudokuState) -> DomElement {
    let clear = bind!(sudoku, || {
        sudoku.clear();
    });

    let example1 = bind!(sudoku, || {
        sudoku.example1();
    });

    let example2 = bind!(sudoku, || {
        sudoku.example2();
    });

    let example3 = bind!(sudoku, || {
        sudoku.example3();
    });

    let css_sudoku_example = css!("
        border: 1px solid black;
        padding: 10px;
        flex-shrink: 1;
        display: flex;
        flex-direction: column;
        margin-right: 10px;
    ");

    let css_sudoku_example_button = css!("
        margin: 5px;
        cursor: pointer;
    ");

    dom! {
        <div css={css_sudoku_example}>
            <button css={css_sudoku_example_button.clone()} on_click={clear}>"Clear"</button>
            <button css={css_sudoku_example_button.clone()} on_click={example1}>"Example 1"</button>
            <button css={css_sudoku_example_button.clone()} on_click={example2}>"Example 2"</button>
            <button css={css_sudoku_example_button} on_click={example3}>"Example 3"</button>
        </div>
    }
}
