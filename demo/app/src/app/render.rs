use vertigo::{
    Computed, Css, DomNode, JsJson, KeyDownEvent, css, dom, dom_element, get_driver,
    include_static, js, transaction,
};

use crate::app::{self, counters::state_counters, state::state_route};

use super::{
    chat::Chat, counters::CountersDemo, driver::DriverDemo, dropfiles::DropFiles, fetch::FetchDemo,
    game_of_life::GameOfLife, github_explorer::GitHubExplorer, home::Home, input::MyInput,
    js_api_access::JsApiAccess, lazy_list::LazyList, list::ListDemo, route::Route,
    styling::Styling, sudoku::Sudoku, svg::SvgDemo, ws_collection::WsCollectionDemo,
};

fn css_menu_item(active: bool) -> Css {
    let bg_color = if active { "lightblue" } else { "lightgreen" };
    css! {"
        display: inline-block;
        padding: 5px 10px;
        cursor: pointer;
        background-color: {bg_color};
        line-height: 30px;

        :hover {
            text-decoration: underline;
        }
    "}
}

fn render_menu_item(menu_item: Route) -> DomNode {
    let css = state_route().route.map({
        let menu_item = menu_item.clone();
        move |current_page| css_menu_item(menu_item == current_page)
    });

    dom! {
        <a
            css={css}
            href={menu_item.to_string()}
        >
            { menu_item.label() }
        </a>
    }
}

/// Whether the keystroke belongs to something the visitor is typing in.
///
/// `hook_key_down` is registered on the document, so it sees every keystroke on the page - the
/// arrows that move a caret through the chat box or the Game Of Life delay field included.
/// Those have to reach the field rather than change tab.
fn is_typing() -> bool {
    match js! { document.activeElement.tagName } {
        JsJson::String(tag) => matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT"),
        _ => false,
    }
}

fn render_header() -> DomNode {
    // Left and right step through the menu. Returning true is what stops the browser also
    // scrolling the page sideways.
    let hook_key_down = |event: KeyDownEvent| {
        let offset = match event.code.as_str() {
            "ArrowRight" => 1,
            "ArrowLeft" => -1,
            _ => return false,
        };

        if is_typing() {
            return false;
        }

        let route = transaction(|context| state_route().route.get(context));
        state_route().set(route.step(offset));
        true
    };

    let css_menu = css! {"
        display: flex;
        flex-wrap: wrap;
        padding: 0;
    "};

    let menu = dom_element! {
        <div css={css_menu} />
    };

    for route in Route::ALL {
        menu.add_child(render_menu_item(route.clone()));
    }

    dom! {
        <div hook_key_down={hook_key_down}>
            { menu }
        </div>
    }
}

fn title_value(state: app::State) -> Computed<String> {
    let sum = state_counters().sum.clone();
    let input_value = state.input.clone();

    Computed::from(move |context| {
        let route = state_route().route.get(context);

        match route {
            Route::Home => "Vertigo demo".into(),
            Route::Counters => {
                let sum = sum.get(context);
                format!("Counter = {sum}")
            }
            Route::Sudoku => "Sudoku".into(),
            Route::Input => {
                let input_value = input_value.get(context);
                format!("Input => {input_value}")
            }
            _ => route.label().to_string(),
        }
    })
}

pub fn render(state: &app::State) -> DomNode {
    let state = state.clone();

    let header = render_header();

    let content = state_route().route.render_value({
        let state = state.clone();

        move |route| match route {
            Route::Home => dom! { <Home /> },
            Route::Styling => dom! { <Styling /> },
            Route::Counters => dom! { <CountersDemo /> },
            Route::Sudoku => dom! { <Sudoku state={&state.sudoku} /> },
            Route::Input => dom! { <MyInput value={&state.input} /> },
            Route::GithubExplorer => dom! { <GitHubExplorer /> },
            Route::GameOfLife => dom! { <GameOfLife state={&state.game_of_life} /> },
            Route::Chat => {
                if let Some(ws_chat) = &state.ws_chat {
                    dom! { <Chat {ws_chat}/> }
                } else {
                    Chat::turn_off_message()
                }
            }
            Route::Fetch => dom! { <FetchDemo /> },
            Route::DropFile => dom! { <DropFiles /> },
            Route::Driver => dom! { <DriverDemo /> },
            Route::JsApiAccess => dom! { <JsApiAccess /> },
            Route::List => dom! { <ListDemo /> },
            Route::LazyList => dom! { <LazyList /> },
            Route::WsCollection => {
                if let Some(ws_collection) = &state.ws_collection {
                    dom! { <WsCollectionDemo {ws_collection}/> }
                } else {
                    WsCollectionDemo::turn_off_message()
                }
            }
            Route::Svg => dom! { <SvgDemo /> },
            Route::NotFound => {
                // Deliberately here rather than somewhere tidier: `set_status` does nothing
                // unless `is_server()`, so it has to run while the server is rendering this
                // route. Rendering is the only thing the server does, which makes the render
                // path the place the status has to be decided.
                get_driver().set_status(404);
                dom! { <div>"Page Not Found"</div> }
            }
        }
    });

    let css_wrapper = css! {"
        padding: 5px;
    "};

    let title_value = title_value(state);

    dom! {
        <html>
            <head>
                <meta charset="utf-8"/>
                <title>{ title_value }</title>
                <link rel="icon" href={include_static!("styling/favicon.png")} />
                <style>"
                    button, input {
                        padding: 2px 8px;
                        margin: 2px;
                        border: 1px solid #333;
                    }
                    button {
                        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
                    }
                    input, textarea {
                        box-shadow: inset 0px 1px 1px rgba(0, 0, 0, 0.2);
                    }
                    hr {
                        margin: 10px 0;
                    }
                "</style>
            </head>
            <body>
                <div css={css_wrapper}>
                    { header }

                    { content }
                </div>
            </body>
        </html>
    }
}
