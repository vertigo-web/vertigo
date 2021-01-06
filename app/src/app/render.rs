use vertigo::{
    computed::Computed,
    VDomNode,
    Css,
    node_attr,
};

use crate::{app};

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

fn render_header(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{build_node, node, css, text, on_click};

    let app_state = app_state.get_value();

    let current_page = &*app_state.route.get_value();

    build_node("div", vec!(
        node("ul", vec!(
            css(css_menu()),
            node("li", vec!(
                text("Main"),
                css(css_menu_item(current_page == &Route::Main)),
                on_click({
                    let app_state = app_state.clone();
                    move || app_state.clone().navigate_to(Route::Main)
                })
            )),
            node("li", vec!(
                text("Counters"),
                css(css_menu_item(current_page == &Route::Counters)),
                on_click({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::Counters)
                })
            )),
            node("li", vec!(
                text("Sudoku"),
                css(css_menu_item(current_page == &Route::Sudoku)),
                on_click({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::Sudoku)
                })
            )),
            node("li", vec!(
                text("Input"),
                css(css_menu_item(current_page == &Route::Input)),
                on_click({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::Input)
                })
            )),
            node("li", vec!(
                text("GitHub Explorer"),
                css(css_menu_item(current_page == &Route::GithubExplorer)),
                on_click({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::GithubExplorer)
                })
            )),
            node("li", vec!(
                text("Game Of Life"),
                css(css_menu_item(current_page.is_game_of_life())),
                on_click({
                    let app_state = app_state.clone();
                    move || {
                        let timer = app_state.game_of_life.get_value().start_timer();
                        let route = Route::GameOfLife {
                            timer
                        };
                        app_state.navigate_to(route);
                    }
                })
            )),
        )),
    ))
}

pub fn render(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{build_node, node, text, component};

    let header = component(app_state.clone(), render_header);

    let app_state = app_state.get_value();

    build_node("div", vec!(
        header,

        match *app_state.route.get_value() {
            Route::Main => {
                component(app_state.main.clone(), super::main::main_render)
            }

            Route::Counters =>
                component(app_state.counters.clone(), super::counters::render),

            Route::Sudoku =>
                node("div", vec!(
                    component(app_state.sudoku.clone(), sudoku::examples_render),
                    component(app_state.sudoku.clone(), sudoku::main_render)
                )),

            Route::Input =>
                component(app_state.input.clone(), input::render),

            Route::GithubExplorer =>
                component(app_state.github_explorer.clone(), github_explorer::render),

            Route::GameOfLife {..} =>
                component(app_state.game_of_life.clone(), game_of_life::render),

            Route::NotFound =>
                node("div", vec!(text("Page Not Found"))),
        }

    ))
}
