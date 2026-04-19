use vertigo::{component, dom};

#[component]
pub fn SvgDemo() {
    dom! {
        <div>
            <svg width="100" height="150">
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
        </div>
    }
}
