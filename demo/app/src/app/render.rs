use vertigo::{Computed, Css, DomNode, KeyDownEvent, css, dom, get_driver, include_static};

use crate::app::{self, counters::state_counters, state::state_route};

use super::{
    chat::Chat, counters::CountersDemo, dropfiles::DropFiles, game_of_life::GameOfLife,
    github_explorer::GitHubExplorer, input::MyInput, js_api_access::JsApiAccess, list::ListDemo,
    route::Route, styling::Styling, sudoku::Sudoku, todo::Todo,
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

fn render_header() -> DomNode {
    let hook_key_down = |event: KeyDownEvent| {
        if event.code == "ArrowRight" {
            log::info!("right");
            return true;
        }

        if event.code == "ArrowLeft" {
            log::info!("left");
            return true;
        }

        false
    };

    let css_menu = css! {"
        display: flex;
        padding: 0;
    "};

    dom! {
        <div hook_key_down={hook_key_down}>
            <div css={css_menu}>
                { render_menu_item(Route::Counters) }
                { render_menu_item(Route::Styling) }
                { render_menu_item(Route::Sudoku) }
                { render_menu_item(Route::Input) }
                { render_menu_item(Route::GithubExplorer) }
                { render_menu_item(Route::GameOfLife) }
                { render_menu_item(Route::Chat) }
                { render_menu_item(Route::Todo) }
                { render_menu_item(Route::DropFile) }
                { render_menu_item(Route::JsApiAccess) }
                { render_menu_item(Route::List) }
            </div>
        </div>
    }
}

fn title_value(state: app::State) -> Computed<String> {
    let sum = state_counters().sum.clone();
    let input_value = state.input.clone();

    Computed::from(move |context| {
        let route = state_route().route.get(context);

        match route {
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
            Route::Todo => dom! { <Todo /> },
            Route::DropFile => dom! { <DropFiles /> },
            Route::JsApiAccess => dom! { <JsApiAccess /> },
            Route::List => dom! { <ListDemo /> },
            Route::NotFound => {
                get_driver().set_status(404);
                dom! { <div>"Page Not Found"</div> }
            }
        }
    });

    let on_keydown = |_event: KeyDownEvent| -> bool { false };

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
            </head>
            <body>
                <div on_key_down={on_keydown} css={css_wrapper}>
                    { header }

                    { content }
                </div>
            </body>
        </html>
    }
}
