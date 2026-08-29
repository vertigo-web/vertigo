use vertigo::{component, dom};

mod animations;
mod images;
pub use animations::Animations;
use images::Images;

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
        <Images />
    }
}
