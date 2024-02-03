use vertigo::{main, DomNode, dom, Value, bind};

mod row;
use row::Row;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Div,
    Div4,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Div => f.write_str("Div"),
            Self::Div4 => f.write_str("Div4")
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct AppState {
    rows: Value<Vec<(String, String)>>,
    mode: Value<Mode>,
}

pub fn app(state: AppState) -> DomNode {
    let AppState { rows, mode } = state;

    let create_rows = bind!(rows, || {
        let new_rows = (1..10_001)
            .map(|i| {
                let i_str = i.to_string();
                (
                    ["row-", i_str.as_str()].concat(),
                    ["Row ", i_str.as_str()].concat(),
                )
            })
            .collect();

        rows.set(new_rows);
    });

    let change_mode = |new_mode: Mode| bind!(mode, || mode.set(new_mode));
    let clear_rows = bind!(rows, || rows.set(vec![]));

    let rows_rendered = mode.render_value(move |mode| match mode {
        Mode::Div =>
            rows.render_list(
                |row| row.0.clone(),
                |(key, label)|
                    dom! {
                        <Row id={&key} label={&label} />
                    }
            ),
        Mode::Div4 =>
            rows.render_list(
                |row| row.0.clone(),
                |(key, label)|
                    dom! {
                        <div>
                            <div>"Row"</div><div>"Label"</div>
                            <Row id={&key} label={&label} />
                        </div>
                    }
            ),
    });

    dom! {
        <html>
            <head />
            <body>
                <div>
                    <div>
                        <button id="generate" on_click={create_rows}>"Generate"</button>
                        <button id="clear" on_click={clear_rows}>"Clear"</button>
                    </div>
                    <div>
                        "Modes: "
                        <button id="mode_div" on_click={change_mode(Mode::Div)}>{Mode::Div}</button>
                        <button id="mode_div4" on_click={change_mode(Mode::Div4)}>{Mode::Div4}</button>
                        "Currently: " {&mode}
                    </div>
                    <div id="row-container">
                        {rows_rendered}
                    </div>
                </div>
            </body>
        </html>
    }
}

#[main]
fn render() -> DomNode {
    let state = AppState {
        rows: Value::new(vec![
            ("row-1".to_string(), "Row 1".to_string()),
            ("row-2".to_string(), "Row 2".to_string()),
            ("row-3".to_string(), "Row 3".to_string()),
        ]),
        mode: Value::new(Mode::Div),
    };
    app(state)
}
