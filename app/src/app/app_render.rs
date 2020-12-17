
use virtualdom::{computed::Computed, vdom::{
        models::{
            VDomNode::VDomNode,
        },
    }};

use super::app_state::AppState;

use virtualdom::vdom::models::{
    Css::Css,
    NodeAttr,
};

use crate::simple_counter::{simple_counter_render::simple_counter_render};
use crate::sudoku::render::{sudoku_render, sudoku_examples_render};

// fn wrapper1() -> Css {
//     Css::new().add("windth: 30px; height: 20px;")
// }

// fn wrapper2(active: bool) -> Css {
//     let mut out = Css::new().add("windth: 30px; height: 20px;");

//     if active {
//         out.str("color: red;");
//     }

//     let url: Option<String> = None;
//     if let Some(url) = url {

//     }

//     out
// }

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

//"border: 1px solid black; padding: 10px; background-color: #e0e0e0;")

/*
    kady statyczny string jest zapisany tylko raz.
    więc kademu statycznemu stringowi będzie odpowiadał jakiś identyfikator
*/

fn render_header(app_state: &Computed<AppState>) -> VDomNode {
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
            node("div", vec!(
                css(cssBg()),
                text("bla bla bla"),
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
    ))
}

// wrapper1();
// wrapper2(true);
// wrapper2(false);

fn render_suma(app_state: &Computed<AppState>) -> VDomNode {
    use NodeAttr::{buildNode, text};

    let app_state = app_state.getValue();

    let suma = app_state.suma.getValue();

    buildNode("div", vec!(
        text(format!("suma = {}", suma))
    ))
}

pub fn main_render(app_state: &Computed<AppState>) -> VDomNode {
    // let counter1 = app_state.map(|app_state| {
    //     let app_state = app_state.getValue().counter1.clone();
    //     let g = *app_state.getValue();
    // });

    /*
        computed<T>
            .map(
                Computed<T> -> Computed<K>
            )

            zwraca finalnie Computed<K>
    */

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
            component(app_state.counter1.clone(), simple_counter_render),
            component(app_state.counter2.clone(), simple_counter_render),
            component(app_state.counter3.clone(), simple_counter_render),
            component(app_state.counter4.clone(), simple_counter_render),
        )),

        suma,

        node("div", vec!(
            component(app_state.sudoku.clone(), sudoku_examples_render),
            component(app_state.sudoku.clone(), sudoku_render)
        ))
    ))
}

    /*
    <div aaa="one" bbb="two">
        "Abudabi"
    </div>
    */
                //.style(wrapper2(true))                //TODO - zaimplementować
                //.style(wrapper1())                    //TODO - zaimplementować

                        // node("div")
        //     .child(node("div")
        //         .css(cssBg())
        //         .child(text("bla bla bla"))
        //     )
        //     .child(node("div")
        //         .onClick(onUp.clone())
        //         .child(text(format!("aktualna wartosc = {} ({})", value, at)))
        //     )
        //     .child(node("div")
        //         .css(cssBg())
        //         .onClick(onUp)
        //         .child(text("up"))
        //     )
        //     .child(node("div")
        //         .css(cssBg())
        //         .onClick(onDown)
        //         .child(text("down"))
        //     )
        //     .child(node("div")
        //         .child(text(format!("jakis footer {} {}", *value % 2, *value % 3)))
        //     ),
        

