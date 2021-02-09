use vertigo::{computed::Computed, VDomElement, Css};
use vertigo_html::{html_component, html_element, css};

use crate::app::sudoku::state::{Cell, number_item::SudokuValue};
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
    css!("
        display: flex;
        align-items: center;
        justify-content: center;
        width: {config.item_width}px;
        height: {config.item_width}px;
        background-color: #00ff00;
        font-size: 40px;
        color: blue;
        cursor: pointer;
    ")
}

fn css_wrapper_one() -> Css {
    let config = Config::new();
    css!("
        width: {config.item_width}px;
        height: {config.item_width}px;
    ")
}

fn css_wrapper() -> Css {
    let config = Config::new();
    css!("
        width: {config.item_width}px;
        height: {config.item_width}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: 1fr 1fr 1fr;
        flex-shrink: 0;
    ")
}

fn css_item(should_show: bool) -> Css {
    let bg_color = if should_show { "#00ff0030" } else { "inherit" };
    let cursor = if should_show { "pointer" } else { "inherit" };

    css!("
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: {bg_color};
        cursor: {cursor};
    ")
}

pub fn render_cell_possible(item: &Computed<Cell>) -> VDomElement {
    let cell = (*item).get_value();

    let possible = (*cell).possible.get_value();
    let only_one_possible = possible.len() == 1;

    if only_one_possible {
        let out: Vec<_> = possible.iter()
            .map(|number| {
                let on_set = {
                    let number = *number;
                    let cell = cell.clone();

                    move || {
                        cell.number.value.set_value(Some(number));
                    }
                };

                html_element!("
                    <div css={css_item_only_one()} onClick={on_set}>
                        { number.to_u16() }
                    </div>
                ")
            })
            .collect();

        return html_component!("
            <div css={css_wrapper_one()}>
                { ..out }
            </div>
        ")
    }

    let possible_last_value = *cell.possible_last.get_value();

    if let Some(possible_last_value) = possible_last_value {
        let on_set = {
            move || {
                cell.number.value.set_value(Some(possible_last_value));
            }
        };

        return html_component!(r#"
            <div css={css_wrapper_one()}>
                <div css={css_item_only_one()} onClick={on_set}>
                    {$ format!("{}.", possible_last_value.to_u16()) $}
                </div>
            </div>
        "#)
    }


    let out: Vec<_> = SudokuValue::variants().into_iter()
        .map(|number| {
            let should_show = possible.contains(&number);

            let label = if should_show {
                format!("{}", number.to_u16())
            } else {
                "".into()
            };

            let on_click = {
                let cell = cell.clone();
                move || {
                    if should_show {
                        cell.number.value.set_value(Some(number));
                    }
                }
            };

            html_element!("
                <div css={css_item(should_show)} onClick={on_click}>
                    { label }
                </div>
            ")
        })
        .collect();

    html_component!("
        <div css={css_wrapper()}>
            { ..out }
        </div>
    ")
}
