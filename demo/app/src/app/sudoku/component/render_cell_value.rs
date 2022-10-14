use vertigo::{css, css_fn, Css, bind, dom, DomElement};

use crate::app::sudoku::state::{number_item::SudokuValue, Cell};

use super::config::Config;

fn css_item_number_wrapper() -> Css {
    let config = Config::new();
    css!(
        "
        position: relative;
        text-align: center;
        font-size: 30px;
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

pub fn render_cell_value(value: SudokuValue, cell: &Cell) -> DomElement {
    let cell = cell.clone();

    let delete_component = cell.show_delete.render_value_option({
        let cell = cell.clone();
        move |show_delete| {
            match show_delete {
                true => {
                    let on_click = bind(&cell).call(|_, cell| {
                        cell.number.value.set(None);
                    });

                    Some(dom! {
                        <div css={css_delete()} on_click={on_click}>
                            "X"
                        </div>
                    })
                },
                false => None,
            }
        }
    });

    dom! {
        <div css={css_item_number_wrapper()}>
            { value.as_u16() }
            {delete_component}
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
