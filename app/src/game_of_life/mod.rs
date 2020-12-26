use std::cmp::PartialEq;
use vertigo::{
    Css,
    NodeAttr,
    VDomNode,
    computed::{
        Computed,
        Dependencies,
        Value
    }
};

fn create_matrix_row(root: &Dependencies, x_count: u16) -> Vec<Computed<Value<bool>>> {
    let mut row = Vec::new();

    for _ in 0..x_count {
        row.push(root.new_value_wrap_width_computed(false));
    }

    row
}

fn create_matrix(root: &Dependencies, x_count: u16, y_count: u16) -> Vec<Vec<Computed<Value<bool>>>> {
    let mut matrix = Vec::new();

    for _ in 0..y_count {
        matrix.push(create_matrix_row(root, x_count));
    }

    matrix
}

#[derive(PartialEq)]
pub struct State {
    pub x_count: Value<u16>,
    pub y_count: Value<u16>,
    pub matrix: Computed<Vec<Vec<Computed<Value<bool>>>>>,
}

impl State {
    pub fn new(root: &Dependencies) -> Computed<State> {
        let x_count = 120;
        let y_count = 80;

        root.new_computed_from(State {
            x_count: root.new_value(x_count),
            y_count: root.new_value(y_count),
            matrix: root.new_computed_from(create_matrix(root, x_count, y_count))
        })
    }
}

fn css_wrapper() -> Css {
    Css::one("
        border: 1px solid black;
        padding: 10px;
        margin: 10px;
        background-color: #e0e0e0;
    ")
}

fn css_row() -> Css {
    Css::one("
        display: flex;
        flex-direction: row;
        height: 10px
    ")
}

fn css_cell(is_active: bool) -> Css {
    let mut css = Css::one("
        width: 10px;
        height: 10px;
        cursor: pointer;
    ");

    if is_active {
        css.str("background-color: black");
    } else {
        css.str("background-color: white");
    }

    css
}

pub fn render(state: &Computed<State>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text};

    let value = state.get_value().matrix.get_value();
    let value_inner = &*value;

    buildNode("div", vec!(
        css(css_wrapper()),
        node("div", vec!(
            text("Game of life 1")
        )),
        render_matrix(value_inner)
    ))
}

fn render_matrix(matrix: &Vec<Vec<Computed<Value<bool>>>>) -> NodeAttr::NodeAttr {
    use NodeAttr::{node};

    let mut out: Vec<NodeAttr::NodeAttr> = Vec::new();

    for item in matrix.iter() {
        out.push(render_row(item));
    }

    node("div", out)
}

fn render_row(matrix: &Vec<Computed<Value<bool>>>) -> NodeAttr::NodeAttr {
    use NodeAttr::{node, css, component};

    let mut out: Vec<NodeAttr::NodeAttr> = Vec::new();

    out.push(css(css_row()));

    for item in matrix.iter() {
        out.push(component(item.clone(), render_cell));
    }

    node("div", out)
}

fn render_cell(cell: &Computed<Value<bool>>) -> VDomNode {
    use NodeAttr::{buildNode, css, onClick};

    let value = cell.get_value();
    let is_active = value.get_value();

    let on_click = {
        let is_active = *is_active;

        move || {
            value.set_value(!is_active);
        }
    };

    buildNode("div", vec!(
        onClick(on_click),
        css(css_cell(*is_active)),
    ))
}
