use alloc::{
    vec,
};
use vertigo::{
    Css,
    node_attr::{NodeAttr, node, css}
};

pub fn spinner() -> NodeAttr {
    node("div", vec!(
        css(Css::one("
            width: 40px;
            height: 40px;
            background-color: #d26913;

            border-radius: 100%;
            animation: 1.0s infinite ease-in-out {
                0% {
                    -webkit-transform: scale(0);
                    transform: scale(0);
                }
                100% {
                    -webkit-transform: scale(1.0);
                    transform: scale(1.0);
                    opacity: 0;
                }
            };
        ")),
    ))
}
