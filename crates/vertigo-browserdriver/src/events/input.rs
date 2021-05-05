use std::rc::Rc;

use wasm_bindgen::{JsCast};

use vertigo::{RealDomId, utils::BoxRefCell};
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement};

use crate::{DomDriverBrowserInner, dom_event::DomEvent};

use super::{find_dom_id, get_from_node};


fn find_event_on_input(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, id: RealDomId) -> Option<Rc<dyn Fn(String)>> {
    get_from_node(
        inner,
        &id,
        |elem| elem.on_input.clone()
    )
}

pub fn create_input_event(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, root: &Element) -> DomEvent {
    let inner = inner.clone();

    DomEvent::new(&root, "input", move |event: web_sys::Event| {

        let dom_id = find_dom_id(&event);
        let event_to_run = find_event_on_input(&inner, dom_id);

        if let Some(event_to_run) = event_to_run {
            let target = event.target().unwrap();
            let input = target.dyn_ref::<HtmlInputElement>();

            if let Some(input) = input {
                event_to_run(input.value());
                return;
            }

            let input = target.dyn_ref::<HtmlTextAreaElement>();

            if let Some(input) = input {
                event_to_run(input.value());
                return;
            }
        }
    })
}