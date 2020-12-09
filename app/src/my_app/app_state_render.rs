
use std::rc::Rc;
use virtualdom::{
    vdom::{
        models::{
            VDomNode::VDomNode,
        },
    },
};

use super::app_state::AppState;

use virtualdom::vdom::models::{
    Css::Css,
    NodeAttr,
};

use crate::simple_counter::{simple_counter_render::simple_counter_render};

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

    // wrapper1();
    // wrapper2(true);
    // wrapper2(false);

pub fn main_render(app_state: Rc<AppState>) -> VDomNode {
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

    let at = app_state.at.getValue();
    let value = app_state.value.getValue();
    let suma = app_state.suma.getValue();

    use NodeAttr::{buildNode, node, css, text, component, onClick, attr};

    let showColor = *value % 2 == 0;

    // let counter2 = app_state.counter2.clone();

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
            node("div", vec!(
                css(cssFooter(showColor)),
                text(format!("jakis footer {} {}", *value % 2, *value % 3)),
            )),
        )),

        node("div", vec!(
            attr("aaa", "one"),
            attr("bbb", "two"),
            text("Abudabi")
        )),

        node("div", vec!(
            component(app_state.counter1.clone(), simple_counter_render),
            component(app_state.counter2.clone(), simple_counter_render),
            component(app_state.counter3.clone(), simple_counter_render),
        )),

        node("div", vec!(
            text(format!("suma = {}", suma))
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
        

