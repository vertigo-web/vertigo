use vertigo::{css, css_fn, html, Computed, Css, VDomElement};

use crate::app::sudoku::state::{number_item::SudokuValue, Cell};

use super::config::Config;

fn css_item_number_wrapper() -> Css {
    let config = Config::new();
    css!(
        "
        position: relative;
        text-align: center;
        font-size: 40px;
        color: blue;
        height: {config.item_width_size}px;
        line-height: {config.item_width_size}px;
    "
    )
}

css_fn! { css_delete, "
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
" }

pub fn render_cell_value(value: SudokuValue, item: &Computed<Cell>) -> VDomElement {
    let cell = item.get_value();

    //cell.show_delete.setValue(true);

    let show_delete = *cell.show_delete.get_value();

    //TODO - Add delete possibility...
    let mut out = Vec::new();

    if show_delete {
        let on_click = move || {
            cell.number.value.set_value(None);
        };

        out.push(
            html! {
                <div css={css_delete()} on_click={on_click}>
                    "X"
                </div>
            }, // node("div", vec!(
               // css(css_delete()),
               // on_click({
               //     move || {
               //         cell.number.value.set_value(None);
               //     }
               // }),
               // text("X")
        );
    }

    html! {
        <div css={css_item_number_wrapper()}>
            { value.as_u16() }
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
