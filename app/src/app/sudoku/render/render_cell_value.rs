use vertigo::{computed::Computed, VDomElement, Css};
use vertigo_html::{Inline, html_component, html_element};

use crate::app::sudoku::state::{Cell, number_item::SudokuValue};

use super::config::Config;

fn css_item_number_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        position: relative;
        text-align: center;
        font-size: 40px;
        color: blue;
        height: {}px;
        line-height: {}px;
    ", config.item_width_size, config.item_width_size))
}

fn css_delete() -> Css {
    Css::one("
        position: absolute;
        top: 3px;
        right: 3px;
        width: 20px;
        height: 20px;
        background-color: #ff000030;
        cursor: pointer;
        font-size: 12px;
        line-height: 12px;

        display: flex;
        align-items: center;
        justify-content: center;
    ")
}



pub fn render_cell_value(value: SudokuValue, item: &Computed<Cell>, ) -> VDomElement {
    let cell = item.get_value();

    //cell.show_delete.setValue(true);

    let show_delete = *cell.show_delete.get_value();

    //TODO - dorobić obsługę delete ...
    let mut out = Vec::new();

    if show_delete {
        let on_click = move || {
            cell.number.value.set_value(None);
        };

        out.push(
            html_element! {
                <div css={css_delete()} onClick={on_click}>
                    X
                </div>
            }
            // node("div", vec!(
            // css(css_delete()),
            // on_click({
            //     move || {
            //         cell.number.value.set_value(None);
            //     }
            // }),
            // text("X")
        );
    }

    html_component! {
        <div css={css_item_number_wrapper()}>
            { value.to_u16() }
            { ..out }
        </div>
    }
}


    //     onMouseEnter = () => {
    //         this.showDelete = true;
    //     }

    //     onMouseOut = () => {
    //         this.showDelete = false;
    //     }

    // }
    //     return (
    //         <ItemNumberWrapper onMouseOver={state.onMouseEnter} onMouseLeave={state.onMouseOut}>
    //         </ItemNumberWrapper>
    //     )
    // })

