
use virtualdom::{
    computed::Computed,
    VDomNode,
    Css,
    NodeAttr,
};

use crate::app;

use crate::simple_counter;
use crate::sudoku;

use super::spinner::spinner;

fn cssFooter(showColor: bool) -> Css {
    let base = Css::one("background-color: yellow;");

    if showColor {
        base.push("color: green;")
    } else {
        base.push("color: blue;")
    }
}

fn cssBg() -> Css {
    Css::one("border: 1px solid black; padding: 10px; background-color: #e0e0e0;margin-bottom: 10px;")
}

fn cssButton() -> Css {
    cssBg().push("cursor: pointer;")
}

fn render_header(app_state: &Computed<app::State>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick};


    let app_state = app_state.getValue();

    let at = app_state.at.getValue();
    let value = app_state.value.getValue();

    let onDown = {
        let app_state = app_state.clone();
        move || {
            app_state.decrement();
        }
    };

    let onUp = {
        let app_state = app_state.clone();
        move || {
            log::info!("on click");
            app_state.increment();
        }
    };

    let showColor = *value % 2 == 0;

    let footer_dom = if *value % 10 == 0 {
        node("div", vec!(
            text(format!("jakis footer {} {} - BEZKLASIE", *value % 2, *value % 3)),
        ))
    } else {
        node("div", vec!(
            css(cssFooter(showColor)),
            text(format!("jakis footer {} {}", *value % 2, *value % 3)),
        ))
    };

    buildNode("div", vec!(
        node("div", vec!(
            css(cssBg()),
            text("bla bla bla"),
            spinner(),
        )),
        node("div", vec!(
            onClick(onUp.clone()),
            text(format!("aktualna wartosc = {} ({})", value, at)),
        )),
        node("div", vec!(
            css(cssButton()),
            onClick(onUp),
            text("up"),
        )),
        node("div", vec!(
            css(cssButton()),
            onClick(onDown),
            text("down"),
        )),
        footer_dom,
    ))
}

fn render_suma(app_state: &Computed<app::State>) -> VDomNode {
    use NodeAttr::{buildNode, text};

    let app_state = app_state.getValue();

    let suma = app_state.suma.getValue();

    buildNode("div", vec!(
        text(format!("suma = {}", suma))
    ))
}

pub fn render(app_state: &Computed<app::State>) -> VDomNode {
    use NodeAttr::{buildNode, node, text, component, attr};

    let header = component(app_state.clone(), render_header);
    let suma = component(app_state.clone(), render_suma);

    let app_state = app_state.getValue();

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
        ))
    ))
}
