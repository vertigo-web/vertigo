use virtualdom::{
    Css, CssFrames,
    NodeAttr::{NodeAttr, node, css, cssFrames}
};

pub fn spinner() -> NodeAttr {
    node("div", vec!(
        css(Css::one("
            width: 40px;
            height: 40px;
            background-color: #d26913;

            border-radius: 100%;
            -webkit-animation: sk-scaleout 1.0s infinite ease-in-out;
            animation: sk-scaleout 1.0s infinite ease-in-out;
        ")),

        cssFrames(CssFrames::new("sk-scaleout", "
            0% {
                -webkit-transform: scale(0);
                transform: scale(0);
            } 100% {
                -webkit-transform: scale(1.0);
                transform: scale(1.0);
                opacity: 0;
            }
        ")),
    ))
}
