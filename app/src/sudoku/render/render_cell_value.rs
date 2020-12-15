use virtualdom::{computed::{Computed::Computed, Value::Value}, vdom::models::VDomNode::VDomNode};

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
    let config = Config::new();
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

    use NodeAttr::{buildNode, node, css, text, onClick, component};

    buildNode("div", vec!(
        css(cssItemNumberWrapper()),
        text(format!("{}", value.to_u16())),
                                                                    //TODO - dorobić obsługę delete ...
    ))
}

    // class State {
    //     showDelete: boolean = false;

    //     constructor(readonly cell: CellType,) {

    //         makeAutoObservable(this);
    //     }

    //     onMouseEnter = () => {
    //         this.showDelete = true;
    //     }

    //     onMouseOut = () => {
    //         this.showDelete = false;
    //     }

    //     onDelete = (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    //         e.stopPropagation();

    //         this.cell.number.value = null;
    //     }
    // }

    // interface ItemNumberPropsType {
    //     cell: CellType,
    //     number: SudokuValue
    // }

    // export const ItemNumber = observer((props: ItemNumberPropsType) => {
    //     const [ state ] = React.useState(() => new State(props.cell));

    //     return (
    //         <ItemNumberWrapper onMouseOver ={state.onMouseEnter} onMouseLeave={state.onMouseOut}>
    //             { props.number }
    //             { state.showDelete ? <Delete title="Delete" onClick={state.onDelete}>X</Delete> : null }
    //         </ItemNumberWrapper>
    //     )
    // })




    // const ItemNumberWrapper = styled('div')`
    //     position: relative;
    //     text-align: center;
    //     font-size: 40px;
    //     color: blue;
    //     height: ${props => props.theme.config.itemWidthSize}px;
    //     line-height: ${props => props.theme.config.itemWidthSize}px;
    // `;

    // const Delete = styled('div')`
    //     position: absolute;
    //     top: 3px;
    //     right: 3px;
    //     width: 20px;
    //     height: 20px;
    //     background-color: #ff000030;
    //     cursor: pointer;
    //     font-size: 12px;
    //     line-height: 12px;

    //     display: flex;
    //     align-items: center;
    //     justify-content: center;
    // `;
