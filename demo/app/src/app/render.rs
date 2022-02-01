use std::rc::Rc;

use vertigo::{css, css_fn, html, Css, VDomNode, VDomElement, VDomComponent};

use crate::app::chat;
use crate::{app, navigate_to};

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

fn render_header(state: &Rc<app::State>) -> VDomElement {
    let route = state.route.get_value();
    let current_page = route.as_ref();

    let navigate_to_gameoflife = {
        let state = state.clone();
        move || {
            state.navigate_to(Route::GameOfLife);
        }
    };

    let is_game_of_life = matches!(current_page, Route::GameOfLife { .. });

    html! {
        <div>
            <ul css={css_menu()}>
                <li css={css_menu_item(current_page == &Route::Main)} on_click={navigate_to!(state, Main)}>"Main"</li>
                <li css={css_menu_item(current_page == &Route::Counters)} on_click={navigate_to!(state, Counters)}>"Counters"</li>
                <li css={css_menu_item(current_page == &Route::Sudoku)} on_click={navigate_to!(state, Sudoku)}>"Sudoku"</li>
                <li css={css_menu_item(current_page == &Route::Input)} on_click={navigate_to!(state, Input)}>"Input"</li>
                <li css={css_menu_item(current_page == &Route::GithubExplorer)} on_click={navigate_to!(state, GithubExplorer)}>"Github Explorer"</li>
                <li css={css_menu_item(is_game_of_life)} on_click={navigate_to_gameoflife}>"Game Of Life"</li>
                <li css={css_menu_item(current_page == &Route::Chat)} on_click={navigate_to!(state, Chat)}>"Chat"</li>
                <li css={css_menu_item(current_page == &Route::Todo)} on_click={navigate_to!(state, Todo)}>"Todo"</li>
            </ul>
        </div>
    }
}

pub fn render(state: Rc<app::State>) -> VDomComponent {
    let header = VDomComponent::new(state.clone(), render_header);

    VDomComponent::new(state, move |state: &Rc<app::State>| -> VDomElement {
        let child: VDomNode = match *state.route.get_value() {
            Route::Main => state.main.clone().into(),
            Route::Counters => state.counters.clone().into(),
            Route::Sudoku => state.sudoku.clone().into(),
            Route::Input => state.input.clone().into(),
            Route::GithubExplorer => state.github_explorer.clone().into(),
            Route::GameOfLife { .. } => state.game_of_life.clone().into(),
            Route::Chat => chat::ChatState::component(&state.driver).into(),
            Route::Todo => super::todo::TodoState::component(&state.driver).into(),
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
