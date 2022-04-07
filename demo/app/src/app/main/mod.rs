use vertigo::{css, css_fn, css_fn_push, html, Css, Driver, VDomElement, Value, VDomComponent};

mod spinner;

use spinner::spinner;

#[derive(Clone)]
pub struct MainState {
    driver: Driver,
    pub value: Value<u32>,
    pub progress: Value<u32>,
}

impl MainState {
    pub fn component(driver: &Driver) -> VDomComponent {
        let state = MainState {
            driver: driver.clone(),
            value: driver.new_value(33),
            progress: driver.new_value(0),
        };

        VDomComponent::new(state, main_render)
    }

    pub fn increment(&self) {
        let rr = self.value.get_value();
        self.value.set_value(*rr + 1);
    }

    pub fn decrement(&self) {
        let rr = self.value.get_value();
        self.value.set_value(*rr - 1);
    }

    pub async fn start_animation(self) {
        for i in 0..50 {
            self.progress.set_value(i as u32);
            self.driver.sleep(100).await;
        }
    }
}

fn css_footer(show_color: bool) -> Css {
    let color = if show_color { "green" } else { "blue" };
    css!(
        "
        background-color: yellow;
        color: { color };
    "
    )
}

css_fn! { css_bg, "
    border: 1px solid black;
    padding: 10px;
    background-color: #e0e0e0;
    margin-bottom: 10px;
" }

css_fn_push! { css_button, css_bg, "
    cursor: pointer;
" }

pub fn main_render(state: &MainState) -> VDomElement {
    let value = *state.value.get_value();

    let on_down = {
        let app_state = state.clone();
        move || {
            app_state.decrement();
        }
    };

    let on_up = {
        let state = state.clone();
        move || {
            log::info!("on click");
            state.increment();
        }
    };

    let show_color = value % 2 == 0;

    let footer_dom = if value % 10 == 0 {
        html! {
            <div>
                "jakis footer" {value % 2} {value % 3} "- BEZKLASIE"
            </div>
        }
    } else {
        html! {
            <div css={css_footer(show_color)}>
                "jakis footer" {value % 2} {value % 3}
            </div>
        }
    };

    let progress = *(state.progress.get_value().as_ref());

    let mut progress_html = Vec::new();

    for _ in 0..progress {
        progress_html.push(html!{
            <span>
                " . "
            </span>
        });
    }

    let on_click_progress = {
        let state = state.clone();
        move || state.clone().start_animation()
    };

    html! {
        <div aaa="one" bbb="two">
            "Abudabi"
            <div css={css_bg()}>
                {$ if value > 35 { "terefere kuku" } else { "bla bla bla" } $}
                { spinner() }
            </div>
            <div css={css_bg()} on_click={on_up.clone()}>
                "Actual value = " { value }
            </div>
            <div css={css_bg()}>
                "Actual value: " { value }
            </div>
            <div css={css_button()} on_click={on_up}>
                "up"
            </div>
            <div css={css_button()} on_click={on_down}>
                "down"
            </div>
            <p>{ footer_dom }</p>

            <button on_click_async={on_click_progress}>
                <span>
                    "start the progress bar"
                </span>
                <span>
                    { ..progress_html }
                </span>
            </button>
        </div>
    }
}
