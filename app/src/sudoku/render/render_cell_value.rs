use virtualdom::{
    computed::Computed,
    vdom::models::VDomNode::VDomNode
};

use crate::sudoku::state::{Cell, number_item::SudokuValue};

use virtualdom::vdom::models::{
    Css::Css
};
use virtualdom::vdom::models::{
    NodeAttr,
};
use super::config::Config;

fn cssItemNumberWrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        position: relative;
        text-align: center;
        font-size: 40px;
        color: blue;
        height: {}px;
        line-height: {}px;
    ", config.itemWidthSize, config.itemWidthSize))
}

fn cssDelete() -> Css {
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
    let cell = item.getValue();

    //cell.show_delete.setValue(true);

    let show_delete = *cell.show_delete.getValue();

    use NodeAttr::{buildNode, node, css, text, onClick};

    let mut out = vec!(
        css(cssItemNumberWrapper()),
        text(format!("{}", value.to_u16())),
    );

    //TODO - dorobić obsługę delete ...

    if show_delete {
        out.push(node("div", vec!(
            css(cssDelete()),
            onClick({
                let cell = cell.clone();
                move || {
                    cell.number.value.setValue(None);
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

