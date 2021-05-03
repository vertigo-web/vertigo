use std::rc::Rc;

use wasm_bindgen::{JsCast};

use vertigo::{RealDomId, utils::BoxRefCell};
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement};

use crate::{DomDriverBrowserInner, dom_event::{DomEvent, DomEventDisconnect}};

use super::find_dom_id;


fn find_event_on_input(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, id: RealDomId) -> Option<Rc<dyn Fn(String)>> {
    let on_input = inner.get_with_context(
        id,
        |state, id| -> Option<Rc<dyn Fn(String)>> {
            let item = state.elements.get(&id).unwrap();

            if let Some(on_input) = &item.on_input {
                return Some(on_input.clone());
            }
            None
        }
    );
    on_input
}

pub fn create_input_event(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, root: &Element) -> DomEventDisconnect {
    let inner = inner.clone();

    DomEvent::new_event(&root, "input", move |event: web_sys::Event| {

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