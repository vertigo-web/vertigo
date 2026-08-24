//! The smallest possible scene: one element with one reactive attribute.
//!
//! Prices what every other workload pays once per operation and cannot avoid - one
//! propagation, one command buffer flush, the JsJson serialization, the wasm to JS round
//! trip, the style element that `addStyles()` re-appends on every bulk update, and one
//! `setAttribute`. Subtracting it from any other per-op figure leaves the marginal DOM work.

use std::rc::Rc;

use vertigo::{DomNode, Value, dom};

pub struct ProbeScene {
    pub class: Value<String>,
    pub classes: [String; 2],
}

pub fn build() -> Rc<ProbeScene> {
    Rc::new(ProbeScene {
        class: Value::new("probe a".to_string()),
        // Equal length, so the two halves of the alternation cost the same.
        classes: ["probe a".to_string(), "probe b".to_string()],
    })
}

pub fn render(scene: Rc<ProbeScene>) -> DomNode {
    dom! {
        <div id="stage-probe" class={scene.class.clone()} />
    }
}
