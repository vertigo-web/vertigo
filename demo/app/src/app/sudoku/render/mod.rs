use vertigo::{css, css_fn, html, Css, VDomElement, VDomComponent, bind};

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

fn render_cell(item: &Cell) -> VDomElement {
    let value = item.number.value.get();

    // log::warn!("cell {:?}", value);
    if let Some(value) = value {
        return render_cell_value::render_cell_value(value, item);
    }

    render_cell_possible::render_cell_possible(item)
}

fn render_group(group: &SudokuSquare<Cell>) -> VDomComponent {
    //log::info!("render group");

    let view1 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::First , TreeBoxIndex::First ), render_cell);
    let view2 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::First , TreeBoxIndex::Middle), render_cell);
    let view3 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::First , TreeBoxIndex::Last  ), render_cell);
    let view4 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First ), render_cell);
    let view5 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle), render_cell);
    let view6 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last  ), render_cell);
    let view7 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::First ), render_cell);
    let view8 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Middle), render_cell);
    let view9 = VDomComponent::from_ref(group.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Last  ), render_cell);

    VDomComponent::from_ref(group, move |_group: &SudokuSquare<Cell>| -> VDomElement {
        html! {
            <div css={css_item_wrapper()}>
                <div css={css_cell_wrapper()}>
                    { view1.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view2.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view3.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view4.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view5.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view6.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view7.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view8.clone() }
                </div>
                <div css={css_cell_wrapper()}>
                    { view9.clone() }
                </div>
            </div>
        }
    })
}

pub fn main_render(sudoku: Sudoku) -> VDomComponent {
    let view1 = VDomComponent::from_ref(&sudoku, examples_render);
    let view2 = main_render_inner(&sudoku);

    VDomComponent::from(sudoku, move |_sudoku: &Sudoku| -> VDomElement {
        html! {
            <div>
                { view1.clone() }
                { view2.clone() }
            </div>
        }
    })
}

pub fn main_render_inner(sudoku: &Sudoku) -> VDomComponent {
    let view1 = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::First ));
    let view2 = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::Middle));
    let view3 = render_group(sudoku.grid.get_from(TreeBoxIndex::First , TreeBoxIndex::Last  ));
    let view4 = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First ));
    let view5 = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle));
    let view6 = render_group(sudoku.grid.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last  ));
    let view7 = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::First ));
    let view8 = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Middle));
    let view9 = render_group(sudoku.grid.get_from(TreeBoxIndex::Last  , TreeBoxIndex::Last  ));

    VDomComponent::from_ref(sudoku, move |_sudoku: &Sudoku| -> VDomElement {
        html! {
            <div css={css_center()}>
                <div css={css_wrapper()}>
                    { view1.clone() }
                    { view2.clone() }
                    { view3.clone() }
                    { view4.clone() }
                    { view5.clone() }
                    { view6.clone() }
                    { view7.clone() }
                    { view8.clone() }
                    { view9.clone() }
                </div>
            </div>
        }
    })
}

css_fn! { css_sudoku_example, "
    border: 1px solid black;
    padding: 10px;
    margin: 10px 0;
" }

css_fn! { css_sudoku_example_button, "
    margin: 5px;
    cursor: pointer;
" }

pub fn examples_render(sudoku: &Sudoku) -> VDomElement {
    let clear = bind(sudoku).call(|sudoku| {
        sudoku.clear();
    });

    let example1 = bind(sudoku).call(|sudoku| {
        sudoku.example1();
    });

    let example2 = bind(sudoku).call(|sudoku| {
        sudoku.example2();
    });

    let example3 = bind(sudoku).call(|sudoku| {
        sudoku.example3();
    });

    html! {
        <div css={css_sudoku_example()}>
            <button css={css_sudoku_example_button()} on_click={clear}>"Clear"</button>
            <button css={css_sudoku_example_button()} on_click={example1}>"Example 1"</button>
            <button css={css_sudoku_example_button()} on_click={example2}>"Example 2"</button>
            <button css={css_sudoku_example_button()} on_click={example3}>"Example 3"</button>
        </div>
    }
}
