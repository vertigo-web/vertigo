use std::{rc::Rc};
use vertigo::{css, css_fn, html, Computed, Css, Driver, VDomElement, Value, VDomComponent, bind};

mod next_generation;

fn create_matrix_row(driver: &Driver, x_count: u16) -> Vec<Value<bool>> {
    let mut row = Vec::new();

    for _ in 0..x_count {
        row.push(driver.new_value(false));
    }

    row
}

fn create_matrix(driver: &Driver, x_count: u16, y_count: u16) -> Vec<Vec<Value<bool>>> {
    let mut matrix = Vec::new();

    for _ in 0..y_count {
        matrix.push(create_matrix_row(driver, x_count));
    }

    matrix
}

#[derive(Clone)]
pub struct State {
    pub driver: Driver,
    pub matrix: Rc<Vec<Vec<Value<bool>>>>,
    pub timer_enable: Value<bool>,
    pub new_delay: Value<u32>,
    pub year: Value<Computed<u32>>,
}

impl State {
    const X_LEN: u16 = 120;
    const Y_LEN: u16 = 70;

    pub fn component(driver: &Driver) -> VDomComponent {
        let matrix = Rc::new(create_matrix(driver, State::X_LEN, State::Y_LEN));

        let timer_enable = driver.new_value(false);
        let new_delay = driver.new_value(150);
        let year = driver.new_value(Self::create_timer(driver, &matrix, &timer_enable, &new_delay, 0));

        let state = State {
            driver: driver.clone(),
            matrix,
            timer_enable,
            new_delay,
            year,
        };

        render(state)
    }

    pub fn accept_new_delay(&self) -> impl Fn() {
        let state = self.clone();
        move ||
            state.year.set_value(
                State::create_timer(&state.driver, &state.matrix, &state.timer_enable, &state.new_delay, *state.year.get_value().get_value())
            )
    }

    pub fn randomize(&self)-> impl Fn() {
        let driver = self.driver.clone();
        let matrix = self.matrix.clone();

        move || {
            log::info!("random ...");

            driver.transaction(|| {
                for (y, row) in matrix.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        let new_value: bool = (y * 2 + (x + 4)) % 2 == 0;
                        cell.set_value(new_value);

                        if x as u16 == State::X_LEN / 2 && y as u16 == State::Y_LEN / 2 {
                            cell.set_value(false);
                        }
                    }
                }
            });
        }
    }

    pub fn create_timer(driver: &Driver, matrix: &Rc<Vec<Vec<Value<bool>>>>, timer_enable: &Value<bool>, new_delay: &Value<u32>, starting_year: u32) -> Computed<u32> {
        let timer_enable = timer_enable.clone();
        let new_delay = *new_delay.get_value();

        driver.new_with_connect(starting_year, {
            let driver = driver.clone();
            let matrix = matrix.clone();

            move |self_value| {
                let driver = driver.clone();
                let timer_enable = timer_enable.clone();
                let self_value = self_value.clone();

                let matrix = matrix.clone();

                log::info!("Setting timer for {} ms", new_delay);

                driver.set_interval(new_delay, {
                    let driver = driver.clone();
                    move || {
                        driver.transaction(|| {
                            let timer_enable = timer_enable.get_value();

                            if *timer_enable {
                                let current = self_value.get_value();
                                self_value.set_value(*current + 1);

                                next_generation::next_generation(&driver, State::X_LEN, State::Y_LEN, &*matrix)
                            }
                        })
                    }
                })
            }
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
    gap: 50px;
    margin-bottom: 5px;
" }

fn render_header(state: &State) -> VDomElement {
    let year = *state.year.get_value().get_value();
    let timer_enable = state.timer_enable.get_value();
    let new_delay = state.new_delay.get_value();

    let button = if *timer_enable {
        let on_click = bind(state).call(|state| {
            state.timer_enable.set_value(false);
            log::info!("stop ...");
        });

        html! {
            <button css={css_button()} on_click={on_click}>
                "Stop"
            </button>
        }
    } else {
        let on_click = bind(state).call(|state| {
            state.timer_enable.set_value(true);
            log::info!("start ...");
        });

        html! {
            <button css={css_button()} on_click={on_click}>
                "Start"
            </button>
        }
    };

    let on_input = bind(state).call_param(|state, new_value: String| {
        state.new_delay.set_value(new_value.parse().unwrap_or_default());
    });

    html! {
        <div css={flex_menu()}>
            <div>"Game of life"</div>
            <div>"Year = " { year }</div>
            <div>
                { button }
                <button css={css_button()} on_click={state.randomize()}>"Random"</button>
            </div>
            <div>
                "Set delay: "
                <input value={new_delay.to_string()} on_input={on_input} />
                " " <button css={css_button()} on_click={state.accept_new_delay()}>"Set"</button>
            </div>
        </div>
    }
}

pub fn render(state: State) -> VDomComponent {

    let view_header = VDomComponent::from_ref(&state, render_header);

    VDomComponent::from(state, move |state: &State| -> VDomElement {
        let matrix = &state.matrix;

        html! {
            <div css={css_wrapper()}>
                { view_header.clone() }
                <br/>
                <a href="https://www.youtube.com/watch?v=C2vgICfQawE" target="_blank">
                    "https://www.youtube.com/watch?v=C2vgICfQawE"
                </a>
                <br/>
                <br/>
                { render_matrix(matrix) }
            </div>
        }
    })
}

fn render_matrix(matrix: &Rc<Vec<Vec<Value<bool>>>>) -> VDomElement {
    let mut out = Vec::new();

    for item in matrix.iter() {
        out.push(render_row(item));
    }

    html! {
        <div>
            { ..out }
        </div>
    }
}

fn render_row(matrix: &[Value<bool>]) -> VDomElement {
    let mut out = Vec::new();

    for item in matrix.iter() {
        out.push(VDomComponent::from_ref(item, render_cell))
    }

    html! {
        <div css={css_row()}>
            { ..out }
        </div>
    }
}

fn render_cell(cell: &Value<bool>) -> VDomElement {
    let is_active = *cell.get_value();

    let on_click_callback = bind(cell)
        .and(&is_active)
        .call(|cell, is_active| {
            cell.set_value(!*is_active);
        });

    html! {
        <div css={css_cell(is_active)} on_click={on_click_callback} />
    }
}
