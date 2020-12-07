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

use virtualdom::vdom::models::{
    Css::Css
};

fn cssBox() -> Css {
    Css::new().add("
        margin: 5px;
    ")
}

fn cssButton() -> Css {
    cssBox().add("
        cursor: pointer;

        &:hover {
            color: red;
        }
    ")
}

fn cssWrapper() -> Css {
    Css::new().add("
        border:1px solid black;
        margin: 5px 0;
    ")
}

pub fn simple_counter_render(simple_counter: Rc<SimpleCounter>) -> Vec<VDom> {
    use NodeAttr::{buildNode, node, css, text, component, onClick, attr};

    let calue =  *(simple_counter.counter.getValue());

    let clickUp = {
        let simple_counter = simple_counter.clone();
        move || {
            simple_counter.increment();
        }
    };

    let clickDown = {
        let simple_counter = simple_counter.clone();
        move || {
            simple_counter.decrement();
        }
    };

    vec!(
        buildNode("div", vec!(
            css(cssWrapper()),
            node("div", vec!(
                css(cssBox()),
                text(format!("Counter value = {}", calue)),
            )),            
            node("button", vec!(
                css(cssButton()),
                text("up"),
                onClick(clickUp)
            )),
            node("button", vec!(
                css(cssButton()),
                text("down"),
                onClick(clickDown)
            ))
        ))
    )
}
