use std::{rc::Rc};
use vertigo::{css, css_fn, Css, Value, bind, get_driver, DomElement, DropResource, dom, transaction};

mod next_generation;

fn create_matrix_row(x_count: u16) -> Vec<Value<bool>> {
    let mut row = Vec::new();

    for _ in 0..x_count {
        row.push(Value::new(false));
    }

    row
}

fn create_matrix(x_count: u16, y_count: u16) -> Vec<Vec<Value<bool>>> {
    let mut matrix = Vec::new();

    for _ in 0..y_count {
        matrix.push(create_matrix_row(x_count));
    }

    matrix
}

#[derive(Clone)]
pub struct State {
    pub matrix: Rc<Vec<Vec<Value<bool>>>>,
    pub timer: Value<Option<Rc<DropResource>>>,             //???
    pub delay: Value<u32>,
    pub new_delay: Value<u32>,
    pub year: Value<u32>,
}

impl State {
    const X_LEN: u16 = 120;
    const Y_LEN: u16 = 70;

    pub fn new() -> State {
        let matrix = Rc::new(create_matrix(State::X_LEN, State::Y_LEN));

        let timer = Value::new(None);
        let delay = Value::new(150);
        let new_delay = Value::new(150);
        let year = Value::new(1);

        State {
            matrix,
            timer,
            delay,
            new_delay,
            year,
        }
    }

    pub fn render(&self) -> DomElement {
        render(self)
    }

    pub fn randomize(&self)-> impl Fn() {
        let matrix = self.matrix.clone();

        move || {
            log::info!("random ...");

            transaction(|_| {
                for (y, row) in matrix.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        let new_value: bool = (y * 2 + (x + 4)) % 2 == 0;
                        cell.set(new_value);

                        if x as u16 == State::X_LEN / 2 && y as u16 == State::Y_LEN / 2 {
                            cell.set(false);
                        }
                    }
                }
            });
        }
    }

    fn start_timer(&self) {
        transaction(|context| {
            let delay = self.delay.get(context);
            let matrix = self.matrix.clone();
            let state = self.clone();

            log::info!("Setting timer for {} ms", delay);

            let timer = get_driver().set_interval(delay, {
                move || {
                    transaction(|context| {
                        let current = state.year.get(context);
                        state.year.set(current + 1);

                        next_generation::next_generation(State::X_LEN, State::Y_LEN, &matrix)
                    })
                }
            });

            self.timer.set(Some(Rc::new(timer)));
        })
    }

    fn accept_new_delay(&self) -> impl Fn() {
        let state = self.clone();

        move || {
            transaction(|context| {
                state.delay.set(state.new_delay.get(context));

                if state.timer.get(context).is_some() {
                    state.start_timer();
                }
            });
        }
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
    css!(
        "
        width: 10px;
        height: 10px;
        cursor: pointer;
        background-color: { color };
    "
    )
}

css_fn! { css_button, "
    cursor: pointer;
" }

css_fn! { flex_menu, "
    display: flex;
    gap: 40px;
    margin-bottom: 5px;
" }

fn render_header(state: &State) -> DomElement {
    let year = state.year.map(|item| {
        item.to_string()
    });
    let delay = state.delay.map(|item| {
        item.to_string()
    });
    let new_delay = state.new_delay.map(|item| {
        item.to_string()
    });

    let on_toggle_timer = {
        let state = state.clone();
        move || {
            transaction(|context| {
                let timer = state.timer.get(context);

                if timer.is_some() {
                    state.timer.set(None);
                } else {
                    state.start_timer();
                }
            });
        }
    };

    let button_label = state.timer.map(|item| -> &'static str {
        match item.is_some() {
            true => "Stop",
            false => "Start",
        }
    });

    let on_input = bind(state).call_param(|_, state, new_value: String| {
        state.new_delay.set(new_value.parse().unwrap_or_default());
    });

    dom! {
        <div css={flex_menu()}>
            <div>
                "Game of life"
            </div>
            <div>
                "Year = "
                <text computed={ year } />
            </div>
            <div>
                <button css={css_button()} on_click={on_toggle_timer}>
                    <text computed={button_label} />
                </button>
                <button css={css_button()} on_click={state.randomize()}>"Random"</button>
            </div>
            <div>
                <div>
                    "delay = "
                    <text computed={delay} />
                </div>
                "Set delay: "
                <input value={new_delay} on_input={on_input} />
                " " <button css={css_button()} on_click={state.accept_new_delay()}>"Set"</button>
            </div>
        </div>
    }
}

pub fn render(state: &State) -> DomElement {

    let matrix = &state.matrix;

    dom! {
        <div css={css_wrapper()}>
            { render_header(state) }
            <br/>
            <a href="https://www.youtube.com/watch?v=C2vgICfQawE" target="_blank">
                "https://www.youtube.com/watch?v=C2vgICfQawE"
            </a>
            <br/>
            <br/>
            { render_matrix(matrix) }
        </div>
    }
}

fn render_matrix(matrix: &Rc<Vec<Vec<Value<bool>>>>) -> DomElement {
    let out = DomElement::new("div");

    for item in matrix.iter() {
        out.add_child(render_row(item));
    }

    out
}

fn render_row(matrix: &[Value<bool>]) -> DomElement {
    let wrapper = dom! {
        <div css={css_row()} />
    };
    
    for item in matrix.iter() {
        wrapper.add_child(render_cell(item));
    }

    wrapper
}

fn render_cell(cell: &Value<bool>) -> DomElement {

    let css_computed = cell.map(css_cell);

    let on_click_callback = bind(cell)
        .call(|context, cell| {
            let is_active = cell.get(context);
            cell.set(!is_active);
        });

        dom! {
            <div on_click={on_click_callback} css={css_computed} />
        }
}
