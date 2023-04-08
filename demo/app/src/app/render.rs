use vertigo::{css, Css, KeyDownEvent, dom, Computed, DomNode};
use crate::app;

use super::{
    dropfiles::DropFiles,
    route::Route,
    animations::Animations,
    counters::CountersDemo,
    game_of_life::GameOfLife,
    github_explorer::GitHubExplorer,
    input::MyInput,
    sudoku::Sudoku,
    chat::Chat,
    todo::Todo,
};

fn css_menu_item(active: bool) -> Css {
    let bg_color = if active { "lightblue" } else { "lightgreen" };
    css!(
        "
        display: inline-block;
        padding: 5px 10px;
        cursor: pointer;
        background-color: {bg_color};
        line-height: 30px;

        :hover {
            text-decoration: underline;
        }
    "
    )
}

fn render_menu_item(current_page: Computed<Route>, menu_item: Route) -> DomNode {
    let css = current_page.map({
        let menu_item = menu_item.clone();
        move |current_page| {
            css_menu_item(menu_item == current_page)
        }
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

fn render_header(state: &app::State) -> DomNode {
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

    let css_menu = css!("
        display: flex;
        margin: 10px;
        padding: 0;
    ");

    dom! {
        <div hook_key_down={hook_key_down}>
            <div css={css_menu}>
                { render_menu_item(state.route.route.clone(), Route::Counters) }
                { render_menu_item(state.route.route.clone(), Route::Animations) }
                { render_menu_item(state.route.route.clone(), Route::Sudoku) }
                { render_menu_item(state.route.route.clone(), Route::Input) }
                { render_menu_item(state.route.route.clone(), Route::GithubExplorer) }
                { render_menu_item(state.route.route.clone(), Route::GameOfLife) }
                { render_menu_item(state.route.route.clone(), Route::Chat) }
                { render_menu_item(state.route.route.clone(), Route::Todo) }
                { render_menu_item(state.route.route.clone(), Route::DropFile) }
            </div>
        </div>
    }
}

fn title_value(state: app::State) -> Computed<String> {
    let sum = state.counters.sum.clone();
    let input_value = state.input.clone();

    Computed::from(move |context| {
        let route = state.route.route.get(context);

        match route {
            Route::Animations => "Animations".into(),
            Route::Counters => {
                let sum = sum.get(context);
                format!("Counter = {sum}")
            },
            Route::Sudoku => "Sudoku".into(),
            Route::Input => {
                let input_value = input_value.get(context);
                format!("Inpput => {input_value}")
            },
            Route::GithubExplorer => "GithubExplorer".into(),
            Route::GameOfLife => "GameOfLife".into(),
            Route::Chat => "Chat".into(),
            Route::Todo => "Todo".into(),
            Route::DropFile => "DropFile".into(),
            Route::NotFound => "NotFound".into(),
        }
    })
}

pub fn render(state: &app::State) -> DomNode {
    let state = state.clone();

    let header = render_header(&state);

    let content = state.route.route.render_value({
        let state = state.clone();

        move |route| {
           match route {
                Route::Animations => dom! { <Animations /> },
                Route::Counters => dom!{ <CountersDemo state={&state.counters} /> },
                Route::Sudoku => dom! { <Sudoku state={&state.sudoku} /> },
                Route::Input => dom! { <MyInput value={&state.input} /> },
                Route::GithubExplorer => dom! { <GitHubExplorer state={&state.github_explorer} /> },
                Route::GameOfLife { .. } => dom! { <GameOfLife state={&state.game_of_life} /> },
                Route::Chat => dom! { <Chat ws_chat={&state.ws_chat}/> },
                Route::Todo => dom! { <Todo /> },
                Route::DropFile => dom! { <DropFiles /> },
                Route::NotFound => dom! { <div>"Page Not Found"</div> },
            }
        }
    });

    let on_keydown = |_event: KeyDownEvent| -> bool {
        false
    };

    let css_wrapper = css!("
        padding: 5px;
    ");

    let title_value = title_value(state);

    dom! {
        <html>
            <head>
                <meta charset="utf-8"/>
                <title>{ title_value }</title>
                <style type="text/css">"
                    * {
                        box-sizing: border-box;
                    }
                    html, body {
                        width: 100%;
                        height: 100%;
                        margin: 0;
                        padding: 0;
                        border: 0;
                    }
                "</style>
            </head>
            <body>
                <div on_key_down={on_keydown} css={css_wrapper}>
                    { header }

                    <div vertigo-suspense={get_css_loading}>
                        "Loading ..."
                    </div>

                    <div vertigo-suspense={get_css_content}>
                        { content }
                    </div>

                </div>
            </body>
        </html>
    }
}

fn get_css_loading(is_loading: bool) -> Css {
    if is_loading {
        css!("
            display: block;
        ")
    } else {
        css!("
            display: none;
        ")
    }
}

fn get_css_content(is_loading: bool) -> Css {
    if is_loading {
        css!("
            display: none;
        ")
    } else {
        css!("
            display: block;
        ")
    }
}
