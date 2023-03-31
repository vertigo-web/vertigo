use vertigo::{bind, dom, DomNode, Value, Computed};

fn is_selected<T: PartialEq + Clone + 'static>(value: &Value<T>, option_value: &T) -> Computed<Option<String>> {
    let option_value = option_value.clone();

    value.to_computed().map(move |current| {
        if current == option_value {
            Some("selected".into())
        } else {
            None
        }
    })
}

pub struct Select<T: Clone> {
    pub value: Value<T>,
    pub options: Computed<Vec<T>>,
}

impl<T> Select<T>
where
    T: Clone + From<String> + PartialEq + ToString + 'static
{
    pub fn mount(&self) -> DomNode {
        let Self { value, options } = self;
        let on_change = bind!(value, |new_value: String| {
            value.set(new_value.into());
        });

        let list = bind!(value, options.render_list(
            |item| item.to_string(),
            move |item| {
                let text_item = item.to_string();
                let selected = is_selected(&value, item);

                dom! { <option value={&text_item} {selected}>{text_item}</option> }
            }
        ));

        dom! {
            <select {on_change}>
                {list}
            </select>
        }
    }
}
