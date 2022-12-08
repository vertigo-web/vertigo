use vertigo::{css, Css, KeyDownEvent, DomElement, dom, Computed};
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
        display: inline;
        width: 60px;
        padding: 5px 10px;
        margin: 5px;
        cursor: pointer;
        background-color: {bg_color};

        :hover {
            text-decoration: underline;
        }
    "
    )
}

fn render_menu_item(current_page: Computed<Route>, menu_item: Route) -> DomElement {
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

fn render_header(state: &app::State) -> DomElement {
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
        list-style-type: none;
        margin: 10px;
        padding: 0;
    ");

    dom! {
        <div hook_key_down={hook_key_down}>
            <ul css={css_menu}>
                { render_menu_item(state.route.route.clone(), Route::Counters) }
                { render_menu_item(state.route.route.clone(), Route::Animations) }
                { render_menu_item(state.route.route.clone(), Route::Sudoku) }
                { render_menu_item(state.route.route.clone(), Route::Input) }
                { render_menu_item(state.route.route.clone(), Route::GithubExplorer) }
                { render_menu_item(state.route.route.clone(), Route::GameOfLife) }
                { render_menu_item(state.route.route.clone(), Route::Chat) }
                { render_menu_item(state.route.route.clone(), Route::Todo) }
                { render_menu_item(state.route.route.clone(), Route::DropFile) }
            </ul>
        </div>
    }
}

pub fn render(state: &app::State) -> DomElement {
    let state = state.clone();

    let header = render_header(&state);

    let content = state.route.route.render_value(
        move |route| {
           match route {
                Route::Animations => dom! { <Animations /> },
                Route::Counters => dom!{ <CountersDemo state={&state.counters} /> },
                Route::Sudoku => dom! { <Sudoku state={&state.sudoku} /> },
                Route::Input => dom! { <MyInput value={&state.input.value} /> },
                Route::GithubExplorer => dom! { <GitHubExplorer state={&state.github_explorer} /> },
                Route::GameOfLife { .. } => dom! { <GameOfLife state={&state.game_of_life} /> },
                Route::Chat => dom! { <Chat /> },
                Route::Todo => dom! { <Todo /> },
                Route::DropFile => dom! { <DropFiles /> },
                Route::NotFound => dom! { <div>"Page Not Found"</div> },
            }
        }
    );

    let on_keydown = |_event: KeyDownEvent| -> bool {
        false
    };

    let css_wrapper = css!("
        padding: 5px;
    ");

    dom! {
        <div on_key_down={on_keydown} css={css_wrapper}>
            { header }
            { content }
        </div>
    }
}
