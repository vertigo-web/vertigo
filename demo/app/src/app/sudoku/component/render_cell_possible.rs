use std::collections::HashSet;

use vertigo::{css, Css, DomElement, dom, Computed, bind2, bind3};
use crate::app::sudoku::state::{number_item::SudokuValue, Cell};

fn css_item_only_one(cell_width: u32) -> Css {
    css!(
        "
        display: flex;
        align-items: center;
        justify-content: center;
        width: {cell_width}px;
        height: {cell_width}px;
        background-color: #00ff00;
        font-size: 30px;
        color: blue;
        cursor: pointer;
    "
    )
}

fn css_wrapper_one(cell_width: u32) -> Css {
    css!(
        "
        width: {cell_width}px;
        height: {cell_width}px;
    "
    )
}

fn css_item(should_show: bool) -> Css {
    let bg_color = if should_show { "#00ff0030" } else { "inherit" };
    let cursor = if should_show { "pointer" } else { "inherit" };

    css!(
        "
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: {bg_color};
        cursor: {cursor};
    "
    )
}

fn view_one_possible(cell_width: u32, cell: &Cell) -> DomElement {
    let cell = cell.clone();

    let render = cell.possible.render_value({
        let cell = cell.clone();
        move |possible| {
            let wrapper = dom! { <div /> };

            for number in possible.iter() {
                let on_set = bind2(&cell, number)
                    .call(|_, cell, number| {
                        cell.number.value.set(Some(*number));
                    });

                wrapper.add_child(dom! {
                    <div css={css_item_only_one(cell_width)} on_click={on_set}>
                        { number.as_u16() }
                    </div>
                });
            }

            wrapper
        }
    });

    dom! {
        <div css={css_wrapper_one(cell_width)}>
            {render}
        </div>
    }
}

fn view_last_value(cell_width: u32, cell: &Cell, possible_last_value: SudokuValue) -> DomElement {
    let on_set = bind2(cell, &possible_last_value)
        .call(|_, cell, possible_last_value| {
            cell.number.value.set(Some(*possible_last_value));
        });

    dom! {
        <div css={css_wrapper_one(cell_width)}>
            <div css={css_item_only_one(cell_width)} on_click={on_set}>
                { possible_last_value.as_u16() }"."
            </div>
        </div>
    }
}

fn view_default(cell_width: u32, cell: &Cell, possible: HashSet<SudokuValue>) -> DomElement {
    let css_wrapper = css!("
        width: {cell_width}px;
        height: {cell_width}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: 1fr 1fr 1fr;
        flex-shrink: 0;
    ");

    let wrapper = dom! { <div css={css_wrapper} /> };

    for number in SudokuValue::variants().into_iter() {
        let should_show = possible.contains(&number);

        let label = if should_show {
            format!("{}", number.as_u16())
        } else {
            "".into()
        };

        let on_click = bind3(cell, &should_show, &number)
            .call(|_, cell, should_show, number| {
                if *should_show {
                    cell.number.value.set(Some(*number));
                }
            });

        wrapper.add_child(dom! {
            <div css={css_item(should_show)} on_click={on_click}>
                { label }
            </div>
        });
    }

    wrapper
}

#[derive(Clone, PartialEq, Eq)]
enum CellView {
    One,
    LastPossible(SudokuValue),
    Default(HashSet<SudokuValue>)
}

pub fn render_cell_possible(cell_width: u32, cell: &Cell) -> DomElement {
    let cell = cell.clone();

    let view = Computed::from({
        let cell = cell.clone();

        move |context| {
            let possible = cell.possible.get(context);
            let only_one_possible = possible.len() == 1;

            if only_one_possible {
                return CellView::One;
            }

            let possible_last_value = cell.possible_last.get(context);

            if let Some(possible_last_value) = possible_last_value {
                return CellView::LastPossible(possible_last_value);
            }

            CellView::Default(possible)
        }
    });

    let render = view.render_value(move |view| {
        match view {
            CellView::One => view_one_possible(cell_width, &cell),
            CellView::LastPossible(last) => view_last_value(cell_width, &cell, last),
            CellView::Default(possible) => view_default(cell_width, &cell, possible),
        }
    });

    dom! {
        <div>
            {render}
        </div>
    }
}
