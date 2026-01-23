use vertigo::{Computed, Value, bind, component, css, dom, dom_element, render::render_list};

fn create_item(val: &str) -> (Value<String>, Computed<String>) {
    let v = Value::new(val.to_string());
    let c = v.map(|str| format!("Computed: {str}"));
    (v, c)
}

#[component]
pub fn ListDemo() {
    let list: Value<Vec<(Value<String>, Computed<String>)>> = Value::new(vec![
        create_item("Item 1"),
        create_item("Item 2"),
        create_item("Item 3"),
    ]);

    let list_computed: Computed<Vec<Computed<String>>> = Computed::from({
        let list = list.clone();
        move |context| {
            let current_list = list.get(context);
            let mut result = Vec::new();
            for (_, comp) in current_list.into_iter() {
                result.push(comp);
            }
            result
        }
    });

    let css_wrapper = css! {"
        display: flex;
        flex-direction: row;
        gap: 20px;
        padding: 20px;
    "};

    let css_panel = css! {"
        flex: 1;
        border: 1px solid #ccc;
        padding: 10px;
    "};

    let on_add = bind!(list, |_| {
        list.change(|current| {
            current.push(create_item("New Item"));
        });
    });

    let left_panel = list.render_value({
        let list = list.clone();
        move |items| {
            let result = dom_element! {
                <div />
            };
            for (item_value, _) in items.into_iter() {
                let on_input = bind!(item_value, |new_value: String| {
                    item_value.set(new_value);
                });

                let on_remove = bind!(list, item_value, |_| {
                    list.change(|current| {
                        current.retain(|(v, _)| v.id() != item_value.id());
                    });
                });

                let css_input = css! {"
                    display: block;
                    padding: 5px;
                    flex: 1;
                    box-sizing: border-box;
                "};

                let css_row = css! {"
                    display: flex;
                    flex-direction: row;
                    gap: 10px;
                    margin-bottom: 5px;
                    align-items: center;
                "};

                let css_btn = css! {"
                    cursor: pointer;
                    padding: 5px 10px;
                "};

                result.add_child(dom! {
                    <div css={css_row}>
                        <input css={css_input} value={item_value} on_input={on_input} />
                        <button css={css_btn} on_click={on_remove}>"Remove"</button>
                    </div>
                });
            }
            result.into()
        }
    });

    let right_panel = render_list(
        &list_computed,
        |item| item.id(),
        |item| {
            log::info!("create item");

            let css_item = css! {"
                margin-bottom: 5px;
                padding: 5px;
                border: 1px solid #eee;
            "};
            dom! {
                <div css={css_item}>
                    {item}
                </div>
            }
        },
    );

    dom! {
        <div css={css_wrapper}>
            <div css={css_panel.clone()}>
                <h3>"Left Panel"</h3>
                <button
                    css={css! {"margin-bottom: 10px; cursor: pointer; padding: 5px 10px;"}}
                    on_click={on_add}
                >
                    "Add Item"
                </button>

                {left_panel}
            </div>

            <div css={css_panel}>
                <h3>"Right Panel"</h3>
                {right_panel}
            </div>
        </div>
    }
}
