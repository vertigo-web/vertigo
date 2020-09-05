
use std::rc::Rc;
use std::collections::HashMap;
use virtualdom::{
    vdom::{
        models::{
            VDom::VDom,
        },
    },
};

use crate::app_state::AppState;

use virtualdom::vdom::models::{
    Css::Css,
    NodeAttr::NodeAttr,
};

fn wrapper1() -> Css {
    Css::new().add("windth: 30px; height: 20px;")
}

fn wrapper2(active: bool) -> Css {
    let mut out = Css::new().add("windth: 30px; height: 20px;");

    if active {
        out.str("color: red;");
    }

    let url: Option<String> = None;
    if let Some(url) = url {
        
    }

    out
}

fn cssBg() -> Css {
    Css::new().add("border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
}

//"border: 1px solid black; padding: 10px; background-color: #e0e0e0;")

/*
    kady statyczny string jest zapisany tylko raz.
    więc kademu statycznemu stringowi będzie odpowiadał jakiś identyfikator
*/

    // wrapper1();
    // wrapper2(true);
    // wrapper2(false);

pub fn main_render(app_state: &Rc<AppState>) -> Vec<VDom> {
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

    vec!(
        NodeAttr::buildNode("div", vec!(
            NodeAttr::node("div", vec!(
                NodeAttr::css(cssBg()),
                NodeAttr::text("bla bla bla"),
            )),
            NodeAttr::node("div", vec!(
                NodeAttr::onClick(onUp.clone()),
                NodeAttr::text(format!("aktualna wartosc = {} ({})", value, at)),
            )),
            NodeAttr::node("div", vec!(
                NodeAttr::css(cssBg()),
                NodeAttr::onClick(onUp),
                NodeAttr::text("up"),
            )),
            NodeAttr::node("div", vec!(
                NodeAttr::css(cssBg()),
                NodeAttr::onClick(onDown),
                NodeAttr::text("down"),
            )),
            NodeAttr::node("div", vec!(
                NodeAttr::text(format!("jakis footer {} {}", *value % 2, *value % 3)),
            )),
        )),

        NodeAttr::buildNode("div", vec!(
            NodeAttr::attr("aaa", "one"),
            NodeAttr::attr("bbb", "two"),
            NodeAttr::text("Abudabi")
        ))
    )
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
        

