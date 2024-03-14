use vertigo::{component, dom};

mod animations;
pub use animations::Animations;

mod spinner;

mod tooltip;
pub use tooltip::TooltipDemo;

#[component]
pub fn Styling() {
    dom! {
        <Animations />
        <TooltipDemo />
    }
}
