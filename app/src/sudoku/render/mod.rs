use virtualdom::{computed::Computed::Computed, vdom::models::VDomNode::VDomNode};
use virtualdom::vdom::models::{
    NodeAttr,
};
use virtualdom::vdom::models::{
    Css::Css
};
use super::state::Sudoku;

fn CssCenter() -> Css {
    Css::one("
        display: flex;
        justify-content: center;
    ")
}

fn CssWrapper() -> Css {
    Css::one("
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: ${props => props.theme.config.allWidth}px;
        height: ${props => props.theme.config.allWidth}px;

        border: 2px solid blue;
    ")
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
