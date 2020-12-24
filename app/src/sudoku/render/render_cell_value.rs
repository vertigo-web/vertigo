use vertigo::{computed::Computed, VDomNode, NodeAttr, Css};

use crate::sudoku::state::{Cell, number_item::SudokuValue};

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



pub fn render_cell_value(value: SudokuValue, item: &Computed<Cell>, ) -> VDomNode {
    let cell = item.get_value();

    //cell.show_delete.setValue(true);

    let show_delete = *cell.show_delete.get_value();

    use NodeAttr::{buildNode, node, css, text, onClick};

    let mut out = vec!(
        css(css_item_number_wrapper()),
        text(format!("{}", value.to_u16())),
    );

    //TODO - dorobić obsługę delete ...

    if show_delete {
        out.push(node("div", vec!(
            css(css_delete()),
            onClick({
                let cell = cell.clone();
                move || {
                    cell.number.value.set_value(None);
                }
            }),
            text("X")
        )));
    }

    buildNode("div", out)
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

