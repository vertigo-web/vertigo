use std::rc::Rc;

use vertigo::{Computed, Css, DomNode, Resource, bind, css, dom, dom_element, transaction};

use super::state::{
    Item, begin_edit, cancel_edit, create_item, delete_item, refresh, save_edit, state_edit_buffer,
    state_editing, state_items, state_new_name, state_status,
};

pub struct LazyList {}

impl LazyList {
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let new_name = state_new_name();

        let on_input_new = bind!(new_name, |v: String| {
            new_name.set(v);
        });
        let on_add = |_| {
            let name = transaction(|ctx| state_new_name().get(ctx));
            create_item(name);
        };
        let on_refresh = |_| {
            refresh();
        };

        let header = dom! {
            <div css={css_section()}>
                <input
                    css={css_input()}
                    placeholder="New item name"
                    value={new_name.to_computed()}
                    on_input={on_input_new}
                />
                <button on_click={on_add}>"Add"</button>
                <button on_click={on_refresh}>"Refresh"</button>
            </div>
        };

        let list = state_items().to_computed().render_value(|res| match res {
            Resource::Loading => dom! { <div>"Loading…"</div> },
            Resource::Error(err) => dom! { <div>"Error: " { err }</div> },
            Resource::Ready(items) => render_list(items),
        });

        let status = state_status().render_value(|status| match status {
            Some(msg) => dom! { <div css={css_error()}>{ msg }</div> },
            None => dom! { <div /> },
        });

        dom! {
            <div css={css_wrapper()}>
                <h2>"LazyListCache CRUD demo"</h2>
                <p css={css_hint()}>
                    "Optimistic create / update / delete with rollback on error. "
                    "Newly created rows briefly show id=0 before the server-assigned id replaces it via update_item_with_old_key."
                </p>
                { header }
                { list }
                { status }
            </div>
        }
    }
}

fn render_list(items: Rc<Vec<Item>>) -> DomNode {
    let wrapper = dom_element! { <div /> };
    for item in items.iter() {
        wrapper.add_child(render_row(item.id));
    }
    wrapper.into()
}

fn render_row(id: u32) -> DomNode {
    let cache = state_items();

    Computed::from(move |ctx| {
        let item = cache.get_by_key(ctx, &id);
        let editing = state_editing().get(ctx) == Some(id);
        (item, editing)
    })
    .render_value(move |(item_res, editing)| match item_res {
        Resource::Loading => {
            dom! { <div css={css_row()}>"row " { id.to_string() } " (pending)"</div> }
        }
        Resource::Error(err) => dom! { <div css={css_row()}>"row error: " { err }</div> },
        Resource::Ready(item) => {
            let item = (*item).clone();
            if editing {
                render_edit_row(item)
            } else {
                render_view_row(item)
            }
        }
    })
}

fn render_view_row(item: Item) -> DomNode {
    let id = item.id;
    let on_edit = bind!(item, |_| {
        begin_edit(&item);
    });
    let on_delete = move |_| {
        delete_item(id);
    };

    let id_label = format!("#{id}");
    dom! {
        <div css={css_row()}>
            <span css={css_id()}>{ id_label }</span>
            <span css={css_name()}>{ item.name }</span>
            <button on_click={on_edit}>"Edit"</button>
            <button on_click={on_delete}>"Delete"</button>
        </div>
    }
}

fn render_edit_row(item: Item) -> DomNode {
    let id = item.id;
    let buffer = state_edit_buffer();
    let on_input = bind!(buffer, |v: String| {
        buffer.set(v);
    });
    let on_save = move |_| {
        let name = transaction(|ctx| state_edit_buffer().get(ctx));
        save_edit(id, name);
    };
    let on_cancel = move |_| {
        cancel_edit(id);
    };

    let id_label = format!("#{id}");
    dom! {
        <div css={css_row()}>
            <span css={css_id()}>{ id_label }</span>
            <input css={css_input()} value={buffer.to_computed()} on_input={on_input} />
            <button on_click={on_save}>"Save"</button>
            <button on_click={on_cancel}>"Cancel"</button>
        </div>
    }
}

fn css_wrapper() -> Css {
    css! {"
        border: 1px solid black;
        margin: 20px 0;
        padding: 10px;
    "}
}

fn css_section() -> Css {
    css! {"
        display: flex;
        gap: 6px;
        align-items: center;
        margin: 8px 0;
    "}
}

fn css_row() -> Css {
    css! {"
        display: flex;
        gap: 6px;
        align-items: center;
        padding: 4px 0;
        border-bottom: 1px solid #ddd;
    "}
}

fn css_id() -> Css {
    css! {"
        min-width: 40px;
        color: #888;
        font-family: monospace;
    "}
}

fn css_name() -> Css {
    css! {"
        flex: 1;
    "}
}

fn css_input() -> Css {
    css! {"
        flex: 1;
        border: 1px solid #333;
        padding: 2px 6px;
    "}
}

fn css_error() -> Css {
    css! {"
        color: #b00;
        margin-top: 8px;
    "}
}

fn css_hint() -> Css {
    css! {"
        color: #555;
        font-size: 0.9em;
    "}
}
