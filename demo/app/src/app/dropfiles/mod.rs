use std::rc::Rc;

use vertigo::{
    Css, DomNode, DropFileEvent, DropFileItem, Value, bind, css, dev::ValueMut, dom,
    render::render_list,
};

pub struct DropFiles {}

#[derive(Clone)]
pub struct DropFilesState {
    /// Each file with an id of its own, so that two files of the same name can both be listed
    /// and removed independently.
    list: Value<Vec<(u32, DropFileItem)>>,
    /// Not a `Value`: nothing renders it, and a write to it should not invalidate anything.
    next_id: Rc<ValueMut<u32>>,
}

impl DropFilesState {
    fn new() -> Self {
        Self {
            list: Value::default(),
            next_id: Rc::new(ValueMut::new(1)),
        }
    }

    /// Take the files from a drop or from the file input - both arrive as a `DropFileEvent`.
    fn add(&self, files: Vec<DropFileItem>) {
        self.list.change(|current| {
            for file in files {
                let id = self.next_id.get();
                self.next_id.set(id + 1);

                log::info!("received {}", describe(&file));
                current.push((id, file));
            }
        });
    }

    fn remove(&self, id: u32) {
        self.list
            .change(|current| current.retain(|(each, _)| *each != id));
    }

    fn clear(&self) {
        self.list.set(Vec::new());
    }
}

impl DropFiles {
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let state = DropFilesState::new();

        let on_dropfile = bind!(state, |event: DropFileEvent| {
            state.add(event.items);
        });

        // The same handler, reached by a route a keyboard or a phone can take - and the only
        // one a WebDriver can drive, since it cannot synthesise a file drop.
        let on_change_file = bind!(state, |event: DropFileEvent| {
            state.add(event.items);
        });

        let on_clear = bind!(state, |_| {
            state.clear();
        });

        let summary = state.list.map(|list| match list.len() {
            0 => "No files yet - drop some below, or choose one above.".to_string(),
            1 => "1 file".to_string(),
            count => format!("{count} files"),
        });

        let list_view = render_list(
            &state.list,
            |(id, _)| *id,
            bind!(state, |entry| {
                entry.render_value(bind!(state, |(id, file)| {
                    let on_remove = bind!(state, id, |_| {
                        state.remove(id);
                    });

                    dom! {
                        <div css={css_row()}>
                            <span>{ describe(&file) }</span>
                            <button on_click={on_remove}>"Remove"</button>
                        </div>
                    }
                }))
            }),
        );

        dom! {
            <div>
                <div css={css_controls()}>
                    <input type="file" multiple="multiple" on_change_file={on_change_file} />
                    <button on_click={on_clear}>"Clear all"</button>
                    <span>{ summary }</span>
                </div>
                <div css={css_drop()} on_dropfile={on_dropfile}>
                    <div>
                        "drop file"
                    </div>
                    <div>
                        { list_view }
                    </div>
                </div>
            </div>
        }
    }
}

fn describe(file: &DropFileItem) -> String {
    let name = &file.name;
    let size = file.data.len();
    format!("{name} - {size} bytes")
}

fn css_controls() -> Css {
    css! {"
        display: flex;
        gap: 10px;
        align-items: center;
        margin: 2px;
    "}
}

fn css_drop() -> Css {
    css! {"
        min-height: 300px;
        background-color: green;
        margin: 2px;
        padding: 5px;
    "}
}

fn css_row() -> Css {
    css! {"
        display: flex;
        gap: 10px;
        align-items: center;
        margin: 2px 0;
    "}
}
