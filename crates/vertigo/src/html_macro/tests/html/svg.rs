use crate::{VDomElement, html};

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn basic() {
    let dom = html! {
        <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M4 1.5C2.89543"
                fill="currentColor"
            />
        </svg>
    };

    let dom2 = VDomElement::build("svg")
        .attr("width", "24")
        .attr("height", "24")
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none")
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .children(
            vec!(
                VDomElement::build("path")
                    .attr("fill-rule", "evenodd")
                    .attr("clip-rule", "evenodd")
                    .attr("d", "M4 1.5C2.89543")
                    .attr("fill", "currentColor")
                    .into()
            )
        )
    ;

    assert_eq!(
        format!("{:?}", dom),
        format!("{:?}", dom2)
    );
}
