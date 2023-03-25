use vertigo::{dom, css, Css, Computed, component};

pub fn light(color: &str) -> Css {
    css!("
        display: inline-block;
        background-color: {color};
        border-radius: 50%;
        width: 100px;
        height: 100px;
        margin: 15px;
    ")
}

#[component]
pub fn Light(light_color: &'static str, dark_color: &'static str, on: Computed<bool>,) {
    let css = on.map(move |on| {
        let color = if on {
            light_color
        } else {
            dark_color
        };

        light(color)
    });

    dom! {
        <div css={css} />
    }
}
