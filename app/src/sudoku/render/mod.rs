use virtualdom::{computed::Computed::Computed, vdom::models::VDomNode::VDomNode};
use virtualdom::vdom::models::{
    NodeAttr,
};
use virtualdom::vdom::models::{
    Css::Css
};
use self::config::Config;

use super::state::{Cell, Sudoku, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

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

fn getGroup(sudoku: &Computed<Sudoku>, x: TreeBoxIndex, y: TreeBoxIndex) -> Computed<SudokuSquare<Cell>> {
    sudoku.clone().map(move |state| {
        state.getValue().grid.getFrom(x, y)
    })
}

fn render_group(group: &Computed<SudokuSquare<Cell>>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick, component};

    buildNode("div", vec!(
        text("grupa"),
    ))
}

pub fn sudoku_render(sudoku: &Computed<Sudoku>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick, component};

    buildNode("div", vec!(
        css(CssCenter()),
        node("div", vec!(
            css(CssWrapper()),
            component(getGroup(sudoku, TreeBoxIndex::First,  TreeBoxIndex::First),  render_group),
            component(getGroup(sudoku, TreeBoxIndex::First,  TreeBoxIndex::Middle), render_group),
            component(getGroup(sudoku, TreeBoxIndex::First,  TreeBoxIndex::Last),   render_group),
            component(getGroup(sudoku, TreeBoxIndex::Middle, TreeBoxIndex::First),  render_group),
            component(getGroup(sudoku, TreeBoxIndex::Middle, TreeBoxIndex::Middle), render_group),
            component(getGroup(sudoku, TreeBoxIndex::Middle, TreeBoxIndex::Last),   render_group),
            component(getGroup(sudoku, TreeBoxIndex::Last,   TreeBoxIndex::First),  render_group),
            component(getGroup(sudoku, TreeBoxIndex::Last,   TreeBoxIndex::Middle), render_group),
            component(getGroup(sudoku, TreeBoxIndex::Last,   TreeBoxIndex::Last),   render_group),
        ))
    ))
}
