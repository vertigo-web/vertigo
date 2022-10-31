use std::rc::Rc;
use vertigo::{css, css_fn, Css, Value, bind, DomElement, dom, transaction};

pub use super::State;

pub struct GameOfLife {
    pub state: State
}

impl GameOfLife {
    pub fn mount(&self) -> DomElement {
        let matrix = &self.state.matrix;

        dom! {
            <div css={css_wrapper()}>
                { Self::render_header(&self.state) }
                <br/>
                <a href="https://www.youtube.com/watch?v=C2vgICfQawE" target="_blank">
                    "https://www.youtube.com/watch?v=C2vgICfQawE"
                </a>
                <br/>
                <br/>
                { Self::render_matrix(matrix) }
            </div>
        }
    }

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

        let on_input = bind!(|state, new_value: String| {
            state.new_delay.set(new_value.parse().unwrap_or_default());
        });

        dom! {
            <div css={flex_menu()}>
                <div>
                    "Game of life"
                </div>
                <div>
                    "Year = " { year }
                </div>
                <div>
                    <button css={css_button()} on_click={on_toggle_timer}>
                        <text computed={button_label} />
                    </button>
                    <button css={css_button()} on_click={state.randomize()}>"Random"</button>
                </div>
                <div>
                    <div>
                        "delay = " {delay}
                    </div>
                    "Set delay: "
                    <input value={new_delay} on_input={on_input} />
                    " " <button css={css_button()} on_click={state.accept_new_delay()}>"Set"</button>
                </div>
            </div>
        }
    }

    fn render_matrix(matrix: &Rc<Vec<Vec<Value<bool>>>>) -> DomElement {
        let out = DomElement::new("div");

        for item in matrix.iter() {
            out.add_child(Self::render_row(item));
        }

        out
    }

    fn render_row(matrix: &[Value<bool>]) -> DomElement {
        let wrapper = dom! {
            <div css={css_row()} />
        };

        for item in matrix.iter() {
            wrapper.add_child(Self::render_cell(item));
        }

        wrapper
    }

    fn render_cell(cell: &Value<bool>) -> DomElement {

        let css_computed = cell.map(css_cell);

        let on_click_callback = bind!(|cell| {
            cell.change(|value| {
                *value = !*value;
            });
        });

        dom! {
            <div on_click={on_click_callback} css={css_computed} />
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
