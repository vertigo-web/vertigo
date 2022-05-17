use vertigo::{DropFileItem, Value, VDomElement, VDomComponent, DropFileEvent, html, Css, css, bind};

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

    pub fn render(self) -> VDomComponent {
        VDomComponent::from(self, render)
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

fn render(state: &DropFilesState) -> VDomElement {
    let on_dropfile = bind(state)
        .call_param(|state, event: DropFileEvent| {

            let mut current = state.list.get();

            for file in event.items.into_iter() {
                let message = format_line(&file);
                log::info!("{message}");

                current.push(file);
            }

            state.list.set(current);
        });

    
    let mut list = Vec::new();

    for file in state.list.get() {
        let message = format_line(&file);
        list.push(html! {
            <div>
                { message }
            </div>
        })
    }
    
    html! {
        <div on_dropfile={on_dropfile} css={css_drop()}>
            <div>
                "drop file"
            </div>
            <div>
                { ..list }
            </div>
        </div>
    }
}

