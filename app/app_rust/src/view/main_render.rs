
use std::rc::Rc;

use virtualdom::{
    vdom::{
        models::{
            VDom::VDom,
        },
    },
};

use crate::app_state::AppState;

pub fn main_render(app_state: &Rc<AppState>) -> Vec<VDom> {
    use virtualdom::vdom::models::{node, text};

    let app_state = app_state.clone();

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
                .onClick(onUp.clone())
                .child(
                    text(format!("aktualna wartosc = {}", value))
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
