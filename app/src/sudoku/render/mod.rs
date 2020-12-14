use virtualdom::{computed::Computed::Computed, vdom::models::VDomNode::VDomNode};
use virtualdom::vdom::models::{
    NodeAttr,
};
use virtualdom::vdom::models::{
    Css::Css
};
use self::config::Config;

use super::state::Sudoku;

pub mod config;

fn CssCenter() -> Css {
    Css::one("
        display: flex;
        justify-content: center;
    ")
}

fn CssWrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: {}px;
        height: {}px;

        border: 2px solid blue;
    ", config.allWidth, config.allWidth))
}

pub fn sudoku_render(sudoku: &Computed<Sudoku>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick};

    buildNode("div", vec!(
        css(CssCenter()),
        node("div", vec!(
            css(CssWrapper()),
            text("sudoku")
        ))
    ))
}
