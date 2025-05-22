use vertigo::{component, dom};

mod animations;
pub use animations::Animations;

mod spinner;

mod tailwind;
use tailwind::Tailwind;

mod tooltip;
pub use tooltip::TooltipDemo;

#[component]
pub fn Styling() {
    dom! {
        <Animations />
        <TooltipDemo />
        <Tailwind />
    }
}
