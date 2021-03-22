use std::cmp::PartialEq;
use vertigo::{
    Css,
    DomDriver,
    VDomElement,
    computed::{
        Computed,
        Dependencies,
        Value
    }
};
use vertigo_html::{html, css, css_fn};

mod next_generation;

fn create_matrix_row(root: &Dependencies, x_count: u16) -> Vec<Value<bool>> {
    let mut row = Vec::new();

    for _ in 0..x_count {
        row.push(root.new_value(false));
    }

    row
}

fn create_matrix(root: &Dependencies, x_count: u16, y_count: u16) -> Vec<Vec<Value<bool>>> {
    let mut matrix = Vec::new();

    for _ in 0..y_count {
        matrix.push(create_matrix_row(root, x_count));
    }

    matrix
}

#[derive(PartialEq)]
pub struct State {
    pub dom_driver: DomDriver,
    pub root: Dependencies,
    pub x_count: Value<u16>,
    pub y_count: Value<u16>,
    pub matrix: Computed<Vec<Vec<Value<bool>>>>,
    pub timer_enable: Value<bool>,
    pub year: Computed<u32>,
}

impl State {
    pub fn new(root: &Dependencies, dom_driver: &DomDriver) -> Computed<State> {
        let x_count_len = 120;
        let y_count_len = 70;

        let x_count = root.new_value(x_count_len);
        let y_count = root.new_value(y_count_len);
        let matrix = root.new_computed_from(create_matrix(root, x_count_len, y_count_len));

        let timer_enable = root.new_value(false);

        let year = {
            let root0 = root.clone();
            let root = root.clone();
            let dom_driver = dom_driver.clone();
            let timer_enable = timer_enable.clone();

            let x_count = x_count.clone();
            let y_count = y_count.clone();
            let matrix = matrix.clone();

            root0.new_with_connect(0, move |self_value| {
                let root = root.clone();
                let timer_enable = timer_enable.clone();
                let self_value = self_value.clone();

                let x_count = x_count.clone();
                let y_count = y_count.clone();
                let matrix = matrix.clone();

                log::info!("start timer");

                let drop_timer = dom_driver.set_interval(1000, move || {

                    let timer_enable = timer_enable.get_value();
        
                    if *timer_enable {
                        let current = self_value.get_value();
                        self_value.set_value(*current + 1);
        
                        let x_count = x_count.get_value();
                        let y_count = y_count.get_value();
                        let matrix = matrix.get_value();
        
                        next_generation::next_generation(&root, *x_count, *y_count, &*matrix)
                    }
                });

                Box::new(drop_timer)
            })
        };


        root.new_computed_from(State {
            dom_driver: dom_driver.clone(),
            root: root.clone(),
            x_count,
            y_count,
            matrix,
            timer_enable,
            year,
        })
    }
}

css_fn! { css_wrapper, "
    border: 1px solid black;
    padding: 10px;
    margin: 10px;
    background-color: #e0e0e0;
" }

css_fn! { css_row, "
    display: flex;
    flex-direction: row;
    height: 10px;
" }

fn css_cell(is_active: bool) -> Css {
    let color = if is_active { "black" } else { "white" };
    css!("
        width: 10px;
        height: 10px;
        cursor: pointer;
        background-color: { color };
    ")
}

css_fn! { css_button, "
    cursor: pointer;
" }

fn render_header(state: &Computed<State>) -> VDomElement {
    let state = state.get_value();
    let year = *state.year.get_value();
    let timer_enable = state.timer_enable.get_value();

    let button = if *timer_enable {
        let on_click = {
            let timer_enable = state.timer_enable.clone();
            move || {
                timer_enable.set_value(false);
                log::info!("stop ...");
            }
        };
        html!("
            <button css={css_button()} onClick={on_click}>
                Stop
            </button>
        ")
    } else {
        let on_click = {
            let timer_enable = state.timer_enable.clone();
            move || {
                timer_enable.set_value(true);
                log::info!("start ...");
            }
        };

        html!("
            <button css={css_button()} onClick={on_click}>
                Start
            </button>
        ")
    };

    let button_random = {
        let on_click = move || {
            log::info!("random ...");

            let x_count = *state.x_count.get_value();
            let y_count = *state.y_count.get_value();

            let matrix = state.matrix.get_value();

            for (y, row) in matrix.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    let new_value: bool = (y * 2 + (x + 4)) % 2 == 0;
                    cell.set_value(new_value);

                    if x as u16 == x_count / 2 && y as u16 == y_count / 2 {
                        cell.set_value(false);
                    }
                }
            }
        };

        html!("
            <button css={css_button()} onClick={on_click}>
                Random
            </button>
        ")
    };

    html!(r#"
        <div>
            <div>Game of life</div>
            <div>year = { year }</div>
            { button }
            { button_random }
        </div>
    "#)
}

pub fn render(state: &Computed<State>) -> VDomElement {
    let value = state.get_value().matrix.get_value();
    let value_inner = &*value;

    html!("
        <div css={css_wrapper()}>
            <component {render_header} data={state.clone()} />
            { render_matrix(value_inner) }
        </div>
    ")
}

fn render_matrix(matrix: &[Vec<Value<bool>>]) -> VDomElement {
    let mut out = Vec::new();

    for item in matrix.iter() {
        out.push(render_row(item));
    }

    html!("
        <div>
            { ..out }
        </div>
    ")
}

fn render_row(matrix: &[Value<bool>]) -> VDomElement {
    let mut out = Vec::new();

    for item in matrix.iter() {
        out.push(
            html!("<component_val {render_cell} data={item} />")
        )
    }

    html!("
        <div css={css_row()}>
            { ..out }
        </div>
    ")
}

fn render_cell(cell: &Value<bool>) -> VDomElement {
    let is_active = cell.get_value();

    let on_click_callback = {
        let cell = cell.clone();
        let is_active = *is_active;

        move || {
            cell.set_value(!is_active);
        }
    };

    html!("
        <div css={css_cell(*is_active)} onClick={on_click_callback} />
    ")
}
