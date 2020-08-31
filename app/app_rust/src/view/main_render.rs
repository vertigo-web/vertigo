
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
    node,
    text,
    Css::Css
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
        node("div")
            .child(node("div")
                .attr("style", "border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
                .child(text("bla bla bla"))
            )
            .child(node("div")
                //.style(wrapper2(true))                //TODO - zaimplementować
                //.style(wrapper1())                    //TODO - zaimplementować
                .onClick(onUp.clone())
                .child(
                    text(format!("aktualna wartosc = {} ({})", value, at))
                )
            )
            .child(node("div")
                .attr("style", "border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
                .child(text("up"))
                .onClick(onUp)
            )
            .child(node("div")
                .attr("style", "border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
                .child(text("down"))
                .onClick(onDown)
            )
            .child(node("div")
                .child(text(format!("jakis footer {} {}", *value % 2, *value % 3)))
            )
    )

    /*
    <div aaa="one" bbb="two">
        "Abudabi"
    </div>
    */
}
