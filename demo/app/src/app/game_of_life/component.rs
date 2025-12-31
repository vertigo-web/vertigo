use std::rc::Rc;
use vertigo::{DomElement, DomNode, Value, bind, css, dom, dom_element};

pub use super::State;

pub struct GameOfLife {
    pub state: State,
}

impl GameOfLife {
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let matrix = &self.state.matrix;
        let css_wrapper = css! {"
            border: 1px solid black;
            padding: 10px;
            margin: 10px;
            background-color: #e0e0e0;
        "};

        dom! {
            <div css={css_wrapper}>
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

    fn render_header(state: &State) -> DomNode {
        let year = state.year.map(|item| item.to_string());
        let delay = state.delay.map(|item| item.to_string());
        let new_delay = state.new_delay.map(|item| item.to_string());

        let button_label = state.timer.map(|item| -> &'static str {
            match item.is_some() {
                true => "Stop",
                false => "Start",
            }
        });

        let on_input = bind!(state, |new_value: String| {
            state.new_delay.set(new_value.parse().unwrap_or_default());
        });

        let flex_menu = css! {"
            display: flex;
            gap: 40px;
            margin-bottom: 5px;
        "};

        let button_css = css! {"
            cursor: pointer;
            border: black 1px solid;
        "};

        let input_css = css! {"
            border: black 1px solid;
        "};

        dom! {
            <div css={flex_menu}>
                <div>
                    "Game of life"
                </div>
                <div>
                    "Year = " { year }
                </div>
                <div>
                    <button css={&button_css} on_click={state.on_toggle_timer()}>
                        {button_label}
                    </button>
                    <button css={&button_css} on_click={state.randomize()}>"Random"</button>
                </div>
                <div>
                    <div>
                        "delay = " {delay}
                    </div>
                    "Set delay: "
                    <input value={new_delay} css={input_css} on_input={on_input} />
                    " " <button css={button_css} on_click={state.accept_new_delay()}>"Set"</button>
                </div>
            </div>
        }
    }

    fn render_matrix(matrix: &Rc<Vec<Vec<Value<bool>>>>) -> DomElement {
        let out = dom_element! {
            <div />
        };

        for item in matrix.iter() {
            out.add_child(Self::render_row(item));
        }

        out
    }

    fn render_row(matrix: &[Value<bool>]) -> DomElement {
        let css_row = css! {"
            display: flex;
            flex-direction: row;
            height: 10px;
        "};

        let wrapper = dom_element! {
            <div css={css_row} />
        };

        for item in matrix.iter() {
            wrapper.add_child(Self::render_cell(item));
        }

        wrapper
    }

    fn render_cell(cell: &Value<bool>) -> DomNode {
        let css_cell = |is_active: bool| {
            let color = if is_active { "black" } else { "white" };
            css! {"
                width: 10px;
                height: 10px;
                cursor: pointer;
                background-color: { color };
            "}
        };

        let css_computed = cell.map(css_cell);

        let on_click_callback = bind!(cell, |_| {
            cell.change(|value| {
                *value = !*value;
            });
        });

        dom! {
            <div on_click={on_click_callback} css={css_computed} />
        }
    }
}
