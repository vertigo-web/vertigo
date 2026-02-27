use vertigo::{DomNode, DropFileEvent, DropFileItem, Value, bind, css, dom, render::render_list};

pub struct DropFiles {}

#[derive(Clone, Default)]
pub struct DropFilesState {
    list: Value<Vec<DropFileItem>>,
}

impl DropFiles {
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let state = DropFilesState::default();

        let list_view = render_list(
            &state.list,
            |item| item.name.clone(),
            |file| {
                let message = format_line(file);
                dom! {
                    <div>
                        { message }
                    </div>
                }
            },
        );

        let on_dropfile = bind!(state, |event: DropFileEvent| {
            state.list.change(|current| {
                for file in event.items.into_iter() {
                    let message = format_line(&file);
                    log::info!("{message}");

                    current.push(file);
                }
            });
        });

        let css_drop = css! {"
            height: 400px;
            background-color: green;
        "};

        dom! {
            <div css={css_drop} on_dropfile={on_dropfile}>
                <div>
                    "drop file"
                </div>
                <div>
                    { list_view }
                </div>
            </div>
        }
    }
}

fn format_line(file: &DropFileItem) -> String {
    let file_name = &file.name;
    let size = file.data.len();
    format!("file name={file_name} size={size}")
}
