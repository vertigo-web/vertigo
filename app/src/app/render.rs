use vertigo::{
    computed::Computed,
    VDomNode,
    Css,
    node_attr,
};

use crate::{app, game_of_life, github_explorer};

use crate::simple_counter;
use crate::sudoku;
use crate::input;

use super::spinner::spinner;
use super::route::Route;

fn css_footer(show_color: bool) -> Css {
    let base = Css::one("background-color: yellow;");

    if show_color {
        base.push("color: green;")
    } else {
        base.push("color: blue;")
    }
}

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

fn css_bg() -> Css {
    Css::one("border: 1px solid black; padding: 10px; background-color: #e0e0e0;margin-bottom: 10px;")
}

fn css_button() -> Css {
    css_bg().push("cursor: pointer;")
}

fn render_header(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{buildNode, node, css, text, onClick};


    let app_state = app_state.get_value();

    let current_page = &*app_state.route.get_value();

    buildNode("div", vec!(
        node("ul", vec!(
            css(css_menu()),
            node("li", vec!(
                text("Main"),
                css(css_menu_item(current_page == &Route::Main)),
                onClick({
                    let app_state = app_state.clone();
                    move || app_state.clone().navigate_to(Route::Main)
                })
            )),
            node("li", vec!(
                text("Counters"),
                css(css_menu_item(current_page == &Route::Counters)),
                onClick({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::Counters)
                })
            )),
            node("li", vec!(
                text("Sudoku"),
                css(css_menu_item(current_page == &Route::Sudoku)),
                onClick({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::Sudoku)
                })
            )),
            node("li", vec!(
                text("Input"),
                css(css_menu_item(current_page == &Route::Input)),
                onClick({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::Input)
                })
            )),
            node("li", vec!(
                text("GitHub Explorer"),
                css(css_menu_item(current_page == &Route::GithubExplorer)),
                onClick({
                    let app_state = app_state.clone();
                    move || app_state.navigate_to(Route::GithubExplorer)
                })
            )),
            node("li", vec!(
                text("Game Of Life"),
                css(css_menu_item(current_page.is_game_of_life())),
                onClick({
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

fn render_suma(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{buildNode, text};

    let app_state = app_state.get_value();

    let suma = app_state.suma.get_value();

    buildNode("div", vec!(
        text(format!("suma = {}", suma))
    ))
}

pub fn render(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{buildNode, css, onClick, node, text, component, attr};

    let header = component(app_state.clone(), render_header);
    let suma = component(app_state.clone(), render_suma);

    let app_state = app_state.get_value();

    buildNode("div", vec!(
        header,


        match *app_state.route.get_value() {
            Route::Main => {
                let value = app_state.value.get_value();

                let on_down = {
                    let app_state = app_state.clone();
                    move || {
                        app_state.decrement();
                    }
                };

                let on_up = {
                    let app_state = app_state.clone();
                    move || {
                        log::info!("on click");
                        app_state.increment();
                    }
                };

                let show_color = *value % 2 == 0;

                let footer_dom = if *value % 10 == 0 {
                    node("div", vec!(
                        text(format!("jakis footer {} {} - BEZKLASIE", *value % 2, *value % 3)),
                    ))
                } else {
                    node("div", vec!(
                        css(css_footer(show_color)),
                        text(format!("jakis footer {} {}", *value % 2, *value % 3)),
                    ))
                };

                node("div", vec!(
                    attr("aaa", "one"),
                    attr("bbb", "two"),
                    text("Abudabi"),
                    node("div", vec!(
                        css(css_bg()),
                        text("bla bla bla"),
                        spinner(),
                    )),
                    node("div", vec!(
                        onClick(on_up.clone()),
                        text(format!("aktualna wartosc = {}", value)),
                    )),
                    node("div", vec!(
                        css(css_button()),
                        onClick(on_up),
                        text("up"),
                    )),
                    node("div", vec!(
                        css(css_button()),
                        onClick(on_down),
                        text("down"),
                    )),
                    footer_dom,
                ))
            }

            Route::Counters =>
                node("div", vec!(
                    component(app_state.counter1.clone(), simple_counter::render),
                    component(app_state.counter2.clone(), simple_counter::render),
                    component(app_state.counter3.clone(), simple_counter::render),
                    component(app_state.counter4.clone(), simple_counter::render),
                    suma,
                )),

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
