use vertigo::{component, css, dom};

#[component]
pub fn TooltipDemo() {
    let popup_css = css! {"
        visibility: hidden;
        width: 120px;
        background-color: black;
        color: #fff;
        text-align: center;
        padding: 5px 5px;
        border-radius: 6px;

        /* Position the tooltip text */
        position: absolute;
        top: -5px;
        left: 10c5%;
    "};

    let label_css = css! {"
        position: relative;
        display: inline-block;
        border-bottom: 1px dotted black;
        margin-top: 30px;

        :hover [popup_css] {
            visibility: visible;
        }
    "};

    dom! {
        <div css={label_css}>
            <span css={popup_css}>"This is content of the tooltip"</span>
            "Label with tooltip"
        </div>
    }
}
