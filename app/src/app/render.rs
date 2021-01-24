use vertigo::{
    computed::Computed,
    VDomElement,
    Css,
    node_attr::component,
};
use vertigo_html::{Inline, html_component, html_element};

use crate::{app, navigate_to};

use super::sudoku;
use super::input;
use super::game_of_life;
use super::github_explorer;
use super::route::Route;

fn css_menu() -> Css {
    Css::one("
        list-style-type: none;
        margin: 10px;
        padding: 0;
    ")
}

fn css_menu_item(active: bool) -> Css {
    Css::new(
        format!("
            display: inline;
            width: 60px;
            padding: 5px 10px;
            margin: 5px;
            cursor: pointer;
            background-color: {};
        ", if active { "lightblue" } else { "lightgreen" })
    )
}

fn render_header(app_state: &Computed<app::State>) -> VDomElement {
    let state = app_state.get_value();

    let current_page = &*state.route.get_value();

    let navigate_to_gameoflife = {
        let state = state.clone();
        move || {
            let timer = state.game_of_life.get_value().start_timer();
            let route = Route::GameOfLife {
                timer
            };
            state.navigate_to(route);
        }
    };

    let is_game_of_life = matches!(current_page, Route::GameOfLife { .. });

    html_component! {
        <div>
            <ul css={css_menu()}>
                <li css={css_menu_item(current_page == &Route::Main)} onClick={navigate_to!(state, Main)}> Main </li>
                <li css={css_menu_item(current_page == &Route::Counters)} onClick={navigate_to!(state, Counters)}> Counters </li>
                <li css={css_menu_item(current_page == &Route::Sudoku)} onClick={navigate_to!(state, Sudoku)}> Sudoku </li>
                <li css={css_menu_item(current_page == &Route::Input)} onClick={navigate_to!(state, Input)}> Input </li>
                <li css={css_menu_item(current_page == &Route::GithubExplorer)} onClick={navigate_to!(state, GithubExplorer)}> Github Explorer </li>
                <li css={css_menu_item(is_game_of_life)} onClick={navigate_to_gameoflife}> Game Of Life </li>
            </ul>
        </div>
    }
}

pub fn render(app_state: &Computed<app::State>) -> VDomElement {
    let state = app_state.get_value();

    let child = match *state.route.get_value() {
        Route::Main => {
            component(state.main.clone(), super::main::main_render)
        }

        Route::Counters =>
            component(state.counters.clone(), super::counters::render),

        Route::Sudoku =>
            html_element! {
                <div>
                    <component {sudoku::examples_render} data={state.sudoku.clone()} />
                    <component {sudoku::main_render} data={state.sudoku.clone()} />
                </div>
            },

        Route::Input =>
            component(state.input.clone(), input::render),

        Route::GithubExplorer =>
            component(state.github_explorer.clone(), github_explorer::render),

        Route::GameOfLife {..} =>
            component(state.game_of_life.clone(), game_of_life::render),

        Route::NotFound =>
            html_element! { <div>Page Not Found</div> },
    };

    html_component! {
        <div>
            <component {render_header} data={app_state.clone()} />
            {child}
        </div>
    }
}
