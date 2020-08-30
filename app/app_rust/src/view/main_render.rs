
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

    let app_state = app_state.clone();

    /*
        zaimplementować FromStr, albo coś takiego
        po to zeby dalo sie umiescic String lub str

        https://docs.rs/maplit/1.0.2/maplit/
    */



    // vec!(
    //     VDom::node("div", map!{
    //         "aaa".into() => "one".into(),
    //         "bbb".into() => format!("'wallll {}'", app_state.at.getValue())
    //     }, vec!(
    //         VDom::text(format!("aktualna wartosc = {}", app_state.value.getValue())),
    //     ))
    // )

    use virtualdom::vdom::models::{node, text};

    let app_state_click = app_state.clone();

    let onDown = {
        let app_state = app_state.clone();
        move || {
            app_state.decrement();
        }
    };

    let onUp = {
        let app_state = app_state.clone();
        move || {
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
                .onClick(move || {
                    log::info!("on click");
                    app_state_click.increment();
                })
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
