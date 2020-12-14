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

fn CssItemWrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        border: {}px solid black;

        width: {}px;
        height: {}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;
    ", config.groupBorderSize, config.groupWidthSize, config.groupWidthSize))
}

fn CssCell() -> Css {
    let config = Config::new();
    Css::new(format!("
        border: {}px solid green;
        width: {}px;
        height: {}px;
    ", config.itemBorderSize, config.itemWidthSize, config.itemWidthSize))
}

fn getGroup(sudoku: &Computed<Sudoku>, x: TreeBoxIndex, y: TreeBoxIndex) -> Computed<SudokuSquare<Cell>> {
    sudoku.clone().map(move |state| {
        state.getValue().grid.getFrom(x, y)
    })
}

fn getCell(group: &Computed<SudokuSquare<Cell>>, x: TreeBoxIndex, y: TreeBoxIndex) -> Computed<Cell> {
    group.clone().map(move |state| {
        state.getValue().getFrom(x, y)
    })
}

fn render_group(group: &Computed<SudokuSquare<Cell>>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick, component};

    buildNode("div", vec!(
        css(CssItemWrapper()),
        component(getCell(group, TreeBoxIndex::First,  TreeBoxIndex::First),  render_cell),
        component(getCell(group, TreeBoxIndex::First,  TreeBoxIndex::Middle), render_cell),
        component(getCell(group, TreeBoxIndex::First,  TreeBoxIndex::Last),   render_cell),
        component(getCell(group, TreeBoxIndex::Middle, TreeBoxIndex::First),  render_cell),
        component(getCell(group, TreeBoxIndex::Middle, TreeBoxIndex::Middle), render_cell),
        component(getCell(group, TreeBoxIndex::Middle, TreeBoxIndex::Last),   render_cell),
        component(getCell(group, TreeBoxIndex::Last,   TreeBoxIndex::First),  render_cell),
        component(getCell(group, TreeBoxIndex::Last,   TreeBoxIndex::Middle), render_cell),
        component(getCell(group, TreeBoxIndex::Last,   TreeBoxIndex::Last),   render_cell),
    ))
}

fn render_cell(item: &Computed<Cell>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick, component};

    buildNode("div", vec!(
        css(CssCell()),
        text("0")                       //TODO
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
