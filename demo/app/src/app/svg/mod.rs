use vertigo::{ClickEvent, Value, bind, component, dom};

#[component]
pub fn SvgDemo() {
    let clicked = Value::new("Click the picture".to_string());
    let on_click = bind!(clicked, |event: ClickEvent| {
        clicked.set(match event.position {
            Some(at) => format!(
                "Clicked at {:.0}, {:.0} of {:.0} × {:.0}",
                at.x, at.y, at.element_width, at.element_height
            ),
            None => "Clicked without a pointer".to_string(),
        });
    });

    dom! {
        <div>
            <svg width="100" height="150" on_click={on_click}>
                <g>
                    <path d="M10 10 L90 10 L90 90 L10 90 Z" fill="none" stroke="black" stroke-width="3" />
                    <circle cx="50" cy="50" r="40" stroke="black" stroke-width="3" fill="red" />
                </g>
                <svg:a href="https://vertigo.znoj.pl">
                    <svg:title>"Go to vertigo.znoj.pl"</svg:title>
                    <path d="M10 100 L90 100 L90 140 L10 140 Z" fill="none" stroke="black" stroke-width="3" />
                    <text x="10" y="130" fill="black">"Link in SVG"</text>
                </svg:a>
            </svg>
            <p>{clicked}</p>
        </div>
    }
}
