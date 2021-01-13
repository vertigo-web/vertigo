use vertigo::{computed::Computed, VDomElement, node_attr, Css};
use vertigo_html::{Inline, html_component, html_element};

use crate::app::sudoku::state::{Cell, number_item::SudokuValue};
use super::config::Config;

// fn cssCell() -> Css {
//     let config = Config::new();
//     Css::new(format!("
//         border: {}px solid green;
//         width: {}px;
//         height: {}px;
//     ", config.itemBorderSize, config.itemWidthSize, config.itemWidthSize))
// }

fn css_item_only_one() -> Css {
    let config = Config::new();
    Css::new(format!("
        display: flex;
        align-items: center;
        justify-content: center;
        width: {}px;
        height: {}px;
        background-color: #00ff00;
        font-size: 40px;
        color: blue;
        cursor: pointer;
    ", config.item_width, config.item_width))
}

fn css_wrapper_one() -> Css {
    let config = Config::new();
    Css::new(format!("
        width: {}px;
        height: {}px;
    ", config.item_width, config.item_width))
}

fn css_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        width: {}px;
        height: {}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: 1fr 1fr 1fr;
        flex-shrink: 0;
    ", config.item_width, config.item_width))
}

fn css_item(should_show: bool) -> Css {
    let mut css = Css::one("
        display: flex;
        align-items: center;
        justify-content: center;
    ");

    if should_show {
        css.str("
            background-color: #00ff0030;
            cursor: pointer;
        ");
    }

    css
}

pub fn render_cell_possible(item: &Computed<Cell>) -> VDomElement {
    use node_attr::css;

    let cell = (*item).get_value();

    let possible = (*cell).possible.get_value();
    let only_one_possible = possible.len() == 1;

    if only_one_possible {
        let mut out = Vec::new();

        out.push(css(css_wrapper_one()));

        for number in possible.iter() {
            let on_set = {
                let number = *number;
                let cell = cell.clone();

                move || {
                    cell.number.value.set_value(Some(number));
                }
            };

            out.push(
                html_element! {
                    <div css={css_item_only_one()} onClick={on_set}>
                        { number.to_u16() }
                    </div>
                }
            );
        }

        return html_component! {
            <div>{ ..out }</div>
        }
    }



    let possible_last_value = *cell.possible_last.get_value();

    if let Some(possible_last_value) = possible_last_value {
        let on_set = {
            move || {
                cell.number.value.set_value(Some(possible_last_value));
            }
        };

        return html_component! {
            <div css={css_wrapper_one()}>
                <div css={css_item_only_one()} onClick={on_set}>
                    {$ format!("{}.", possible_last_value.to_u16()) $}
                </div>
            </div>
        }
    }


    let mut out = Vec::new();
    out.push(css(css_wrapper()));

    for number in SudokuValue::variants() {
        let should_show = possible.contains(&number);

        let label = if should_show {
            format!("{}", number.to_u16())
        } else {
            "".into()
        };

        out.push({
            let on_click = {
                let cell = cell.clone();
                move || {
                    if should_show {
                        cell.number.value.set_value(Some(number));
                    }
                }
            };
            html_element! {
                <div css={css_item(should_show)} onClick={on_click}>
                    { label }
                 </div>
            }
        });
    }

    html_component! {
        <div>{ ..out }</div>
    }
}
