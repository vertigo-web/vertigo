use vertigo::{css, css_fn, Css, KeyDownEvent, DomElement, dom, Computed, bind2};
use crate::app;
use crate::app::chat::ChatState;
use crate::app::todo::TodoState;
use super::dropfile::DropFilesState;
use super::route::Route;

css_fn! { css_menu, "
    list-style-type: none;
    margin: 10px;
    padding: 0;
" }

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

fn navigate_to(state: &app::State, route: Route) -> impl Fn() {
    bind2(state, &route).call(|_, state, route| {
        state.navigate_to(route.clone())
    })
}

fn render_menu_item(state: &app::State, current_page: Computed<Route>, menu_item: Route) -> DomElement {
    let css = current_page.map({
        let menu_item = menu_item.clone();
        move |current_page| {
            css_menu_item(menu_item == current_page)
        }
    });

    dom! {
        <li
            css={css}
            on_click={navigate_to(state, menu_item.clone())}
        >
            { menu_item.label() }
        </li>
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

    dom! {
        <div hook_key_down={hook_key_down}>
            <ul css={css_menu()}>
                { render_menu_item(state, state.route.route.clone(), Route::Counters) }
                { render_menu_item(state, state.route.route.clone(), Route::Animations) }
                { render_menu_item(state, state.route.route.clone(), Route::Sudoku) }
                { render_menu_item(state, state.route.route.clone(), Route::Input) }
                { render_menu_item(state, state.route.route.clone(), Route::GithubExplorer) }
                { render_menu_item(state, state.route.route.clone(), Route::GameOfLife) }
                { render_menu_item(state, state.route.route.clone(), Route::Chat) }
                { render_menu_item(state, state.route.route.clone(), Route::Todo) }
                { render_menu_item(state, state.route.route.clone(), Route::DropFile) }
            </ul>
        </div>
    }
}

pub fn render(state: app::State) -> DomElement {
    let header = render_header(&state);

    let content = state.route.route.render_value(
        move |route| {
           match route {
                Route::Animations => state.animations.render(),
                Route::Counters => state.counters.render(),
                Route::Sudoku => state.sudoku.render(),
                Route::Input => state.input.render(),
                Route::GithubExplorer => state.github_explorer.render(),
                Route::GameOfLife { .. } => state.game_of_life.render(),
                Route::Chat => ChatState::new().render(),
                Route::Todo => TodoState::new().render(),
                Route::DropFile => DropFilesState::new().render(),
                Route::NotFound => dom! { <div>"Page Not Found"</div> },
            }
        }
    );

    let on_keydown = |_event: KeyDownEvent| -> bool {
        false
    };

    dom! {
        <div on_key_down={on_keydown}>
            { header }
            { content }
        </div>
    }
}
