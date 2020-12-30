
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

fn css_footer(show_color: bool) -> Css {
    let base = Css::one("background-color: yellow;");

    if show_color {
        base.push("color: green;")
    } else {
        base.push("color: blue;")
    }
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

    let at = app_state.at.get_value();
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

    buildNode("div", vec!(
        node("div", vec!(
            css(css_bg()),
            text("bla bla bla"),
            spinner(),
        )),
        node("div", vec!(
            onClick(on_up.clone()),
            text(format!("aktualna wartosc = {} ({})", value, at)),
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

fn render_suma(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{buildNode, text};

    let app_state = app_state.get_value();

    let suma = app_state.suma.get_value();

    buildNode("div", vec!(
        text(format!("suma = {}", suma))
    ))
}

pub fn render(app_state: &Computed<app::State>) -> VDomNode {
    use node_attr::{buildNode, node, text, component, attr};

    let header = component(app_state.clone(), render_header);
    let suma = component(app_state.clone(), render_suma);

    let app_state = app_state.get_value();

    buildNode("div", vec!(
        header,

        node("div", vec!(
            attr("aaa", "one"),
            attr("bbb", "two"),
            text("Abudabi")
        )),

        node("div", vec!(
            component(app_state.counter1.clone(), simple_counter::render),
            component(app_state.counter2.clone(), simple_counter::render),
            component(app_state.counter3.clone(), simple_counter::render),
            component(app_state.counter4.clone(), simple_counter::render),
        )),

        suma,

        node("div", vec!(
            component(app_state.sudoku.clone(), sudoku::examples_render),
            component(app_state.sudoku.clone(), sudoku::main_render)
        )),

        component(app_state.input.clone(), input::render),

        component(app_state.github_explorer.clone(), github_explorer::render),

        component(app_state.game_of_life.clone(), game_of_life::render),
    ))
}
