use vertigo::{DomElement, dom, css, Css, Computed};

pub struct Light<'a> {
    pub light_color: &'a str,
    pub dark_color: &'a str,
    pub on: Computed<bool>,
}

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

impl<'a> Light<'a> {
    pub fn mount(&self) -> DomElement {
        let light_color = self.light_color.to_string();
        let dark_color = self.dark_color.to_string();

        let css = self.on.map(move |on| {
            let color = if on {
                light_color.clone()
            } else {
                dark_color.clone()
            };

            light(&color)
        });

        dom! {
            <div css={css} />
        }
    }
}
