use super::state::State;

use vertigo::{
    computed::Computed,
    VDomNode,
    NodeAttr,
    Css
};

fn css_box() -> Css {
    Css::one("
        margin: 5px;
    ")
}

fn css_button() -> Css {
    css_box().push("
        cursor: pointer;

        &:hover {
            color: red;
        }
    ")
}

fn css_wrapper() -> Css {
    Css::one("
        border:1px solid black;
        margin: 5px 0;
    ")
}

pub fn render(simple_counter: &Computed<State>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, onClick};

    let simple_counter = simple_counter.get_value();
    let calue = *(simple_counter.counter.get_value());

    let click_up = {
        let simple_counter = simple_counter.clone();
        move || {
            simple_counter.increment();
        }
    };

    let click_down = {
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
        css(css_wrapper()),
        node("div", vec!(
            css(css_box()),
            text(format!("Counter value = {}", calue)),
        )),
        node("button", vec!(
            css(css_button()),
            text("up"),
            onClick(click_up)
        )),
        node("button", vec!(
            css(css_button()),
            text("down"),
            onClick(click_down)
        ))
    ))
}
