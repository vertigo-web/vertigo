use vertigo::{Computed, Css, DomNode, bind, component, css, dom};

use super::state::{KINDS, Ukulele, WsCollectionState};

#[component]
pub fn WsCollectionDemo(ws_collection: String) {
    let state = WsCollectionState::new(ws_collection);

    let controls = render_controls(&state);
    let table = render_table(&state);

    dom! {
        <div>
            <p>
                "A server-pushed reactive collection (" <code>"vertigo::WsCollection"</code> "). "
                "The server sends an initial snapshot, then streams live updates: the "
                <b>"stock"</b> " column ticks on its own, and rows occasionally disappear and "
                "come back — all without any user action. Use the controls to change the query."
            </p>
            { controls }
            { table }
        </div>
    }
}

impl WsCollectionDemo {
    pub fn turn_off_message() -> DomNode {
        dom! {
            <div>
                <p>"WS Collection demo is turned off."</p>
                <p>"To use it, run the demo locally. After cloning the vertigo repository, run:"</p>
                <p><pre>"cargo make demo"</pre></p>
            </div>
        }
    }
}

fn render_controls(state: &WsCollectionState) -> DomNode {
    let on_kind = bind!(state, |new_kind: String| {
        state.set_kind(new_kind);
    });

    let on_search = bind!(state, |text: String| {
        state.set_search(text);
    });

    let selected_kind = state.kind.to_computed();
    let search_value = state.search.to_computed();

    let options = {
        let mut out = vec![render_option("", "All kinds", &selected_kind)];
        for kind in KINDS {
            out.push(render_option(kind, kind, &selected_kind));
        }
        out
    };

    let row_css = css! {"
        display: flex;
        gap: 10px;
        align-items: center;
        margin-bottom: 10px;
    "};

    dom! {
        <div css={row_css}>
            <label>"Type: "</label>
            <select on_change={on_kind}>
                { ..options }
            </select>
            <label>"Name search: "</label>
            <input type="text" placeholder="e.g. Koa" value={search_value} on_input={on_search} />
        </div>
    }
}

fn render_option(value: &str, label: &str, selected_kind: &Computed<String>) -> DomNode {
    let value = value.to_string();
    let label = label.to_string();
    let selected = selected_kind.map({
        let value = value.clone();
        move |current| {
            if current == value {
                Some("selected".to_string())
            } else {
                None
            }
        }
    });

    dom! { <option value={&value} {selected}>{label}</option> }
}

fn render_table(state: &WsCollectionState) -> DomNode {
    state
        .collection
        .items_sorted
        .render_value(|rows| match rows {
            None => dom! { <div>"Loading…"</div> },
            Some(items) => render_rows(items),
        })
}

fn cell_css() -> Css {
    css! {"
        border: 1px solid #ccc;
        padding: 4px 8px;
        text-align: left;
    "}
}

fn header_cell_css() -> Css {
    css! {"
        border: 1px solid #ccc;
        padding: 4px 8px;
        text-align: left;
        background-color: #f0f0f0;
    "}
}

fn render_rows(items: Vec<Ukulele>) -> DomNode {
    let table_css = css! {"
        border-collapse: collapse;
    "};

    if items.is_empty() {
        return dom! { <div>"No ukuleles match the current query."</div> };
    }

    let head = ["Name", "Type", "Scale (in)", "Tuning", "Stock"]
        .into_iter()
        .map(|title| dom! { <th css={header_cell_css()}>{title}</th> })
        .collect::<Vec<_>>();

    let body = items.into_iter().map(render_row).collect::<Vec<_>>();

    dom! {
        <table css={table_css}>
            <thead>
                <tr>
                    { ..head }
                </tr>
            </thead>
            <tbody>
                { ..body }
            </tbody>
        </table>
    }
}

fn render_row(item: Ukulele) -> DomNode {
    let scale = format!("{}", item.scale_inches);
    let stock = format!("{}", item.stock);
    dom! {
        <tr>
            <td css={cell_css()}>{item.name}</td>
            <td css={cell_css()}>{item.kind}</td>
            <td css={cell_css()}>{scale}</td>
            <td css={cell_css()}>{item.tuning}</td>
            <td css={cell_css()}>{stock}</td>
        </tr>
    }
}
