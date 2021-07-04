use std::rc::Rc;

use wasm_bindgen::{JsCast};

use vertigo::{RealDomId};
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement};

use crate::dom_driver_browser::{DomDriverBrowser};
use crate::dom_event::DomEvent;


fn find_event_on_input(dom_driver: &DomDriverBrowser, id: RealDomId) -> Option<Rc<dyn Fn(String)>> {
    dom_driver.get_from_node(
        &id,
        |elem| elem.on_input.clone()
    )
}

pub fn create_input_event(dom_driver: &DomDriverBrowser, root: &Element) -> DomEvent {
    let dom_driver = dom_driver.clone();

    DomEvent::new(&root, "input", move |event: web_sys::Event| {

        let dom_id = dom_driver.find_dom_id(&event);
        let event_to_run = find_event_on_input(&dom_driver, dom_id);

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