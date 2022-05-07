use vertigo::{css, css_fn, html, Css, VDomNode, VDomElement, VDomComponent, bind};

use crate::app::chat;
use crate::{app};

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

        :nth-of-type(1):hover {
            color: crimson;
            text-decoration: line-through;
        };
    "
    )
}

fn navigate_to(state: &app::State, route: Route) -> impl Fn() {
    bind(state)
        .and(&route)
        .call(|state, route| {
            state.navigate_to(route.clone())
        })
}


fn render_header(state: &app::State) -> VDomElement {
    let route = state.route.get_value();
    let current_page = route.as_ref();

    let is_game_of_life = matches!(current_page, Route::GameOfLife { .. });

    html! {
        <div>
            <ul css={css_menu()}>
                <li
                    css={css_menu_item(current_page == &Route::Main)}
                    on_click={navigate_to(state, Route::Main)}
                >
                    "Main"
                </li>
                <li
                    css={css_menu_item(current_page == &Route::Counters)}
                    on_click={navigate_to(state, Route::Counters)}
                >
                    "Counters"
                </li>
                <li
                    css={css_menu_item(current_page == &Route::Sudoku)}
                    on_click={navigate_to(state, Route::Sudoku)}
                >
                    "Sudoku"
                </li>
                <li
                    css={css_menu_item(current_page == &Route::Input)}
                    on_click={navigate_to(state, Route::Input)}
                >
                    "Input"
                </li>
                <li
                    css={css_menu_item(current_page == &Route::GithubExplorer)}
                    on_click={navigate_to(state, Route::GithubExplorer)}
                >
                    "Github Explorer"
                </li>
                <li
                    css={css_menu_item(is_game_of_life)}
                    on_click={navigate_to(state, Route::GameOfLife)}
                >
                    "Game Of Life"
                </li>
                <li
                    css={css_menu_item(current_page == &Route::Chat)}
                    on_click={navigate_to(state, Route::Chat)}
                >
                    "Chat"
                </li>
                <li
                    css={css_menu_item(current_page == &Route::Todo)}
                    on_click={navigate_to(state, Route::Todo)}
                >
                    "Todo"
                </li>
            </ul>
        </div>
    }
}

pub fn render(state: app::State) -> VDomComponent {
    let header = VDomComponent::from_ref(&state, render_header);

    VDomComponent::from(state, move |state: &app::State| -> VDomElement {
        let child: VDomNode = match *state.route.get_value() {
            Route::Main => state.main.clone().into(),
            Route::Counters => state.counters.clone().into(),
            Route::Sudoku => state.sudoku.clone().into(),
            Route::Input => state.input.clone().into(),
            Route::GithubExplorer => state.github_explorer.clone().into(),
            Route::GameOfLife { .. } => state.game_of_life.clone().into(),
            Route::Chat => chat::ChatState::component().into(),
            Route::Todo => super::todo::TodoState::component().into(),
            Route::NotFound => html! { <div>"Page Not Found"</div> }.into(),
        };

        html! {
            <div>
                { header.clone() }
                {child}
            </div>
        }
    })
}
