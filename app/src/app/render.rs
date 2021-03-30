use vertigo::{
    computed::Computed,
    VDomElement,
    Css,
};
use vertigo_html::{html, css, css_fn};

use crate::{app, navigate_to};

use super::sudoku;
use super::input;
use super::game_of_life;
use super::github_explorer;
use super::route::Route;

css_fn! { css_menu, "
    list-style-type: none;
    margin: 10px;
    padding: 0;
" }

fn css_menu_item(active: bool) -> Css {
    let bg_color = if active { "lightblue" } else { "lightgreen" };
    css!("
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
    ")
}

fn render_header(app_state: &Computed<app::State>) -> VDomElement {
    let state = app_state.get_value();

    let current_page = &*state.route.get_value();

    let navigate_to_gameoflife = {
        let state = state.clone();
        move || {
            state.navigate_to(Route::GameOfLife);
        }
    };

    let is_game_of_life = matches!(current_page, Route::GameOfLife { .. });

    html!("
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
    ")
}

pub fn render(app_state: &Computed<app::State>) -> VDomElement {
    let state = app_state.get_value();

    let child = match *state.route.get_value() {
        Route::Main =>
            html!("<component {super::main::main_render} data={state.main} />"),

        Route::Counters =>
            html!("<component {super::counters::render} data={state.counters} />"),

        Route::Sudoku =>
            html!("
                <div>
                    <component {sudoku::examples_render} data={state.sudoku.clone()} />
                    <component {sudoku::main_render} data={state.sudoku.clone()} />
                </div>
            ").into(),

        Route::Input =>
            html!("<component {input::render} data={state.input} />"),

        Route::GithubExplorer =>
            html!("<component {github_explorer::render} data={state.github_explorer} />"),

        Route::GameOfLife {..} =>
            html!("<component {game_of_life::render} data={state.game_of_life} />"),

        Route::NotFound =>
            html!("<div>Page Not Found</div>").into(),
    };

    html!("
        <div>
            <component {render_header} data={app_state.clone()} />
            {child}
        </div>
    ")
}
