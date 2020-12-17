use super::simple_counter_state::SimpleCounter;

use virtualdom::{computed::Computed, vdom::{
        models::{
            VDomNode::VDomNode,
        },
    }};

use virtualdom::vdom::models::{
    NodeAttr,
};

use virtualdom::vdom::models::{
    Css::Css
};

fn cssBox() -> Css {
    Css::one("
        margin: 5px;
    ")
}

fn cssButton() -> Css {
    cssBox().push("
        cursor: pointer;

        &:hover {
            color: red;
        }
    ")
}

fn cssWrapper() -> Css {
    Css::one("
        border:1px solid black;
        margin: 5px 0;
    ")
}

pub fn simple_counter_render(simple_counter: &Computed<SimpleCounter>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick};

    let simple_counter = simple_counter.getValue();
    let calue = *(simple_counter.counter.getValue());

    let clickUp = {
        let simple_counter = simple_counter.clone();
        move || {
            simple_counter.increment();
        }
    };

    let clickDown = {
        move || {
            simple_counter.decrement();
        }
    };

    // html! {
    //     <div css={cssWrapper()}>
    //         <div css={cssBox()}>{ format!("Counter value = {}", calue) }</div>
    //         <div css={cssButton()} onClick={clickUp}>up</div>
    //         <div css={cssButton()} onClick={clickDown}>down</div>
    //     </div>
    // }

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
}
