use vertigo::{css, Value, bind, dom, DomElement};

#[derive(Clone, Default)]
pub struct MyInput {
    pub value: Value<String>,
}

impl MyInput {
    pub fn mount(&self) -> DomElement {

        let value = &self.value;

        let on_set1 = bind!(value, || {
            value.set("value 1".into());
        });

        let on_set2 = bind!(value, || {
            value.set("value 2".into());
        });

        let on_set3 = bind!(value, |new_value: String| {
            value.set(new_value);
        });

        let on_set4 = bind!(value, |new_value: String| {
            value.set(new_value);
        });

        let mouse_in = || {
            log::info!("enter");
        };

        let mouse_out = || {
            log::info!("out");
        };

        let value = self.value.to_computed();

        let count = value.map(|inner| inner.len().to_string());

        let wrapper = css!("
            border: 1px solid black;
            margin: 20px 0;
            padding: 10px;
        ");

        let input_css = css!("
            margin-left: 10px;
        ");

        let button_css = || css!("
            margin: 0 10px;
            cursor: pointer;
        ");

        let text_css = css!("
            width: 600px;
            height: 300px;
            border: 1px solid black;
            padding: 5px;
            margin: 10px;
        ");

        dom! {
            <div css={wrapper} on_mouse_enter={mouse_in} on_mouse_leave={mouse_out}>
                "This is input"
                <input css={input_css} value={value.clone()} on_input={on_set3} />
                <button css={button_css()} on_click={on_set1}>"set 1"</button>
                <button css={button_css()} on_click={on_set2}>"set 2"</button>
                <textarea css={text_css} on_input={on_set4} value={value} />
                <div>
                    "count = " {count}
                </div>
            </div>
        }
    }
}
