
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

    //node(format!("node aaa='{}' bbb='{}'", "one", format!("'wallll {}'", app_state.at.getValue())))
    vec!(
        node("div")
            .attr("aaa", "one")
            .attr("bbb", format!("'wallll {}'", app_state.at.getValue()))
            .onClick(move || {
                log::info!("on click");
                app_state_click.increment();
            })
            .child(
                text(format!("aktualna wartosc = {}", app_state.value.getValue()))
            )
    )
    

    // node("node")
    //     .attr("aaa", "one")
    //     .attr("bbb", format!("'wallll {}'", app_state.at.getValue()))
    //     .onClick(||{
    //         log::info!("on click")
    //     })
    //     .child(node("a")
    //         .attr("href", "dsadsa")
    //         .attr("blob", "dsadsa")
    //         .onClick(|| {

    //         })
    //         .child(text("link do czegos"))
    //     )
    //     .child(node("div"))
    //     .child(text("bla bla bla"))





    // VDom::node("node")
    //     .attr("aaa", "one")
    //     .attr("bbb", format!("'wallll {}'", app_state.at.getValue())
    //     .onClick(||{

    //     })
    //     .child(vec!(
    //         VDom::node("a")
    //             .attr("href", "dsadsa")
    //             .attr("blob", "dsadsa")
    //             .onClick(|| {

    //             })
    //             .child(vec!(
    //                 VDom::text("link do czegos")
    //             )),
    //         VDom::node("div"),
    //         VDom::text("bla bla bla")
    //     ))

    // vec!(
    //     VDom::node("div", map!{
    //         "aaa".into() => "one".into(),
    //         "bbb".into() => format!("'wallll {}'", app_state.at.getValue())
    //     }, vec!(
    //         VDom::Text {
    //             node: VDomText {
    //                 value: format!("aktualna wartosc = {}", app_state.value.getValue()),
    //             }
    //         }
    //     ))
    // )

    // VDom::Node {
    //     node: VDomNode {
    //         name: "div".into(),
    //         attr: map!{ "aaa".into() => "one".into(), "bbb".into() => format!("'wallll {}'", app_state.at.getValue()) },
    //         child: vec!(
    //             VDom::Text {
    //                 node: VDomText {
    //                     value: format!("aktualna wartosc = {}", app_state.value.getValue()),
    //                 }
    //             }
    //         )
    //     }
    // }

    /*
    <div aaa="one" bbb="two">
        "Abudabi"
    </div>
    */
}
