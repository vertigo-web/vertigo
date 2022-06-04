use vertigo::{DropFileItem, Value, DropFileEvent, Css, css, bind, dom, DomElement};

#[derive(Clone)]
pub struct DropFilesState {
    list: Value<Vec<DropFileItem>>,
}

impl DropFilesState {
    pub fn new() -> DropFilesState {
        DropFilesState {
            list: Value::new(Vec::new())
        }
    }

    pub fn render(self) -> DomElement {
        render(self)
    }
}


fn css_drop() -> Css {
    css!("
        height: 400px;
        background-color: green;
    ")
}

fn format_line(file: &DropFileItem) -> String {
    let file_name = &file.name;
    let size = file.data.len();
    format!("file name={file_name} size={size}")
}

fn render(state: DropFilesState) -> DomElement {
    let list_view = state.list.render_list(
        |item| item.name.clone(),
        |file| {
            let message = format_line(file);
            dom! {
                <div>
                    { message }
                </div>
            }
        }
    );
    
    let on_dropfile = bind(&state)
        .call_param(|context, state, event: DropFileEvent| {
            let mut current = state.list.get(context);

            for file in event.items.into_iter() {
                let message = format_line(&file);
                log::info!("{message}");

                current.push(file);
            }

            state.list.set(current);
        });

    dom! {
        <div css={css_drop()} on_dropfile={on_dropfile}>
            <div>
                "drop file"
            </div>
            <div>
                { list_view }
            </div>
        </div>
    }
}

