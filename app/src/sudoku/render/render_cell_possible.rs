use virtualdom::{computed::Computed::Computed, vdom::models::VDomNode::VDomNode};

use crate::sudoku::state::{Cell, number_item::SudokuValue};
use virtualdom::vdom::models::{
    NodeAttr,
};
use virtualdom::vdom::models::{
    Css::Css
};
use super::config::Config;


// fn cssCell() -> Css {
//     let config = Config::new();
//     Css::new(format!("
//         border: {}px solid green;
//         width: {}px;
//         height: {}px;
//     ", config.itemBorderSize, config.itemWidthSize, config.itemWidthSize))
// }

fn cssItemOnlyOne() -> Css {
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
    ", config.itemWidth, config.itemWidth))
}

fn cssWrapperOne() -> Css {
    let config = Config::new();
    Css::new(format!("
        width: {}px;
        height: {}px;
    ", config.itemWidth, config.itemWidth))
}

fn cssWrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        width: {}px;
        height: {}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: 1fr 1fr 1fr;
        flex-shrink: 0;
    ", config.itemWidth, config.itemWidth))
}

fn cssItem(shouldShow: bool) -> Css {
    let mut css = Css::one("
        display: flex;
        align-items: center;
        justify-content: center;
    ");

    if shouldShow {
        css.str("
            background-color: #00ff0030;
            cursor: pointer;
        ");
    }

    css
}

pub fn render_cell_possible(item: &Computed<Cell>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick};

    let cell = (*item).getValue();

    let possible = (*cell).possible.getValue();
    let onlyOnePossible = possible.len() == 1;

    if onlyOnePossible {
        let mut out = Vec::new();
        
        out.push(css(cssWrapperOne()));

        for number in possible.iter() {
            let onSet = {
                let number = number.clone();
                let cell = cell.clone();

                move || {
                    cell.number.value.setValue(Some(number));
                }
            };
    
            out.push(
                node("div", vec!(
                    css(cssItemOnlyOne()),
                    onClick(onSet),
                    text(format!("{}", number.to_u16()))
                ))
            );
        }
    
        return buildNode("div", out);
    }



    let possibleLastValue = *cell.possibleLast.getValue();

    if let Some(possibleLastValue) = possibleLastValue {
        let onSet = {
            let possibleLastValue = possibleLastValue.clone();
            let cell = cell.clone();

            move || {
                cell.number.value.setValue(Some(possibleLastValue));
            }
        };

        return buildNode("div", vec!(
            css(cssWrapperOne()),
            node("div", vec!(
                css(cssItemOnlyOne()),
                onClick(onSet),
                text(format!("{}.", possibleLastValue.to_u16()))
            ))
        ));
    }


    let mut out = Vec::new();
    out.push(css(cssWrapper()));

    for number in SudokuValue::variants() {
        let shouldShow = possible.contains(&number);

        let label = if shouldShow {
            format!("{}", number.to_u16())
        } else {
            "".into()
        };
        
        out.push(
            node("div", vec!(
                css(cssItem(shouldShow)),
                onClick({
                    let cell = cell.clone();
                    move || {
                        if shouldShow {
                            cell.number.value.setValue(Some(number));
                        }
                    }
                }),
                text(label)
            ))
        );
    }

    buildNode("div", out)
}
