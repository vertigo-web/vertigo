use std::rc::Rc;
use vertigo::{DomElement, DomNode, Value, bind, css, dom, dom_element};

pub use super::State;
use super::patterns;

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
                { Self::render_patterns(&self.state) }
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

    /// The preset boards, one button each, plus the two that are not patterns.
    fn render_patterns(state: &State) -> DomNode {
        let row_css = css! {"
            display: flex;
            gap: 5px;
            align-items: center;
            flex-wrap: wrap;
            margin-bottom: 5px;
        "};

        let button_css = css! {"
            cursor: pointer;
            border: black 1px solid;
        "};

        let out = dom_element! {
            <div css={row_css}>
                <div>"Start from: "</div>
            </div>
        };

        for pattern in patterns::PATTERNS {
            out.add_child(dom! {
                <button
                    css={&button_css}
                    title={pattern.about}
                    on_click={state.load_pattern(pattern)}
                >
                    {pattern.name}
                </button>
            });
        }

        out.add_child(dom! {
            <button css={&button_css} on_click={state.randomize()}>"Random"</button>
        });
        out.add_child(dom! {
            <button css={button_css} on_click={state.clear()}>"Clear"</button>
        });

        out.into()
    }

    fn render_header(state: &State) -> DomNode {
        let year = state.year.map(|item| item.to_string());
        let delay = state.delay.map(|item| item.to_string());
        let new_delay = state.new_delay.to_computed();

        let button_label = state.timer.map(|item| -> &'static str {
            match item.is_some() {
                true => "Stop",
                false => "Start",
            }
        });

        // Stored as typed. Validation happens once, when Set is pressed, so that a
        // half-finished entry is not rewritten under the caret.
        let on_input = bind!(state, |new_value: String| {
            state.new_delay.set(new_value);
        });

        let delay_error = state.delay_error.render_value_option(|error| {
            error.map(|message| {
                let css_error = css! {"
                    color: #b00;
                "};

                dom! { <div css={css_error}>"Delay not set: " { message }</div> }
            })
        });

        let population = state.population.map(|count| count.to_string());

        let flex_menu = css! {"
            display: flex;
            gap: 40px;
            flex-wrap: wrap;
            margin-bottom: 5px;
        "};

        let button_css = css! {"
            cursor: pointer;
            border: black 1px solid;
        "};

        let input_css = css! {"
            border: black 1px solid;
        "};

        // Both of these change every generation, and a spaceship makes them change *width* -
        // an LWSS breathes between 9 and 12 cells. Without room reserved for the widest value
        // they will ever hold, the flex row re-lays-out on every frame and everything to the
        // right of them - Start, Step, the delay field - slides about while the board runs.
        let css_counter = |width: &str| {
            css! {"
                min-width: {width};
            "}
        };

        dom! {
            <div css={flex_menu}>
                <div>
                    "Game of life"
                </div>
                <div css={css_counter("14ch")}>
                    "Year = " { year }
                </div>
                <div css={css_counter("18ch")}>
                    "Population = " { population }
                </div>
                <div>
                    <button css={&button_css} on_click={state.on_toggle_timer()}>
                        {button_label}
                    </button>
                    <button css={&button_css} on_click={state.step()}>"Step"</button>
                </div>
                <div>
                    <div>
                        "delay = " {delay}
                    </div>
                    "Set delay: "
                    <input value={new_delay} css={input_css} on_input={on_input} />
                    " " <button css={button_css} on_click={state.accept_new_delay()}>"Set"</button>
                    { delay_error }
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
