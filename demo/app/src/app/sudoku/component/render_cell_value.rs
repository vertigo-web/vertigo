use vertigo::{bind, css, dom, DomNode};

use crate::app::sudoku::state::{number_item::SudokuValue, Cell};

pub fn render_cell_value(item_height_size: u32, value: SudokuValue, cell: &Cell) -> DomNode {
    let cell = cell.clone();

    let delete_component = cell.show_delete.render_value_option({
        let cell = cell.clone();
        move |show_delete| match show_delete {
            true => {
                let on_click = bind!(cell, |_| {
                    cell.number.set(None);
                });

                let css_delete = css! {"
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
                "};

                Some(dom! {
                    <div css={css_delete} on_click={on_click}>
                        "X"
                    </div>
                })
            }
            false => None,
        }
    });

    let css_item = css! {"
        position: relative;
        text-align: center;
        font-size: 30px;
        color: blue;
        height: {item_height_size}px;
        line-height: {item_height_size}px;
    "};

    dom! {
        <div css={css_item}>
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
