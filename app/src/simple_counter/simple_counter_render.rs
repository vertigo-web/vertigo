use super::simple_counter::SimpleCounter;

use std::rc::Rc;
use virtualdom::{
    vdom::{
        models::{
            VDom::VDom,
        },
    },
};

use virtualdom::vdom::models::{
    NodeAttr,
};

pub fn simple_counter_render(simple_counter: &Rc<SimpleCounter>) -> Vec<VDom> {
    use NodeAttr::{buildNode, node, css, text, component, onClick, attr};

    let calue =  *(simple_counter.counter.getValue());

    let click = {
        let simple_counter = simple_counter.clone();
        move || {
            simple_counter.increment();
        }
    };

    vec!(
        buildNode("div", vec!(
            text("Counter"),
            text(format!("value = {}", calue)),
            onClick(click)
        ))
    )
}
