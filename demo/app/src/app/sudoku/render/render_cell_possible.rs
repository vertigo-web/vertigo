use vertigo::{css, html, Css, VDomElement, bind};

use crate::app::sudoku::state::{number_item::SudokuValue, Cell};

use super::config::Config;

// fn cssCell() -> Css {
//     let config = Config::new();
//     css!("
//         border: {config.item_border_size}px solid green;
//         width: {config.item_width_size}px;
//         height: {config.item_width_size}px;
//     ")
// }

fn css_item_only_one() -> Css {
    let config = Config::new();
    css!(
        "
        display: flex;
        align-items: center;
        justify-content: center;
        width: {config.item_width}px;
        height: {config.item_width}px;
        background-color: #00ff00;
        font-size: 40px;
        color: blue;
        cursor: pointer;
    "
    )
}

fn css_wrapper_one() -> Css {
    let config = Config::new();
    css!(
        "
        width: {config.item_width}px;
        height: {config.item_width}px;
    "
    )
}

fn css_wrapper() -> Css {
    let config = Config::new();
    css!(
        "
        width: {config.item_width}px;
        height: {config.item_width}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: 1fr 1fr 1fr;
        flex-shrink: 0;
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

pub fn render_cell_possible(cell: &Cell) -> VDomElement {
    let possible = (*cell).possible.get_value();
    let only_one_possible = possible.len() == 1;

    if only_one_possible {
        let out = possible.iter().map(|number| {
            let on_set = bind(cell)
                .and(number)
                .call(|cell, number| {
                    cell.number.value.set_value(Some(*number));
                });

            html! {
                <div css={css_item_only_one()} on_click={on_set}>
                    { number.as_u16() }
                </div>
            }
        });

        return html! {
            <div css={css_wrapper_one()}>
                { ..out }
            </div>
        };
    }

    let possible_last_value = *cell.possible_last.get_value();

    if let Some(possible_last_value) = possible_last_value {
        let on_set = bind(cell)
            .and(&possible_last_value)
            .call(|cell, possible_last_value| {
                cell.number.value.set_value(Some(*possible_last_value));
            });

        return html! {
            <div css={css_wrapper_one()}>
                <div css={css_item_only_one()} on_click={on_set}>
                    { possible_last_value.as_u16() }"."
                </div>
            </div>
        };
    }

    let out = SudokuValue::variants().into_iter().map(|number| {
        let should_show = possible.contains(&number);

        let label = if should_show {
            format!("{}", number.as_u16())
        } else {
            "".into()
        };

        let on_click = bind(cell)
            .and(&should_show)
            .and(&number)
            .call(|cell, should_show, number| {
                if *should_show {
                    cell.number.value.set_value(Some(*number));
                }
            });

        html! {
            <div css={css_item(should_show)} on_click={on_click}>
                { label }
            </div>
        }
    });

    html! {
        <div css={css_wrapper()}>
            { ..out }
        </div>
    }
}
