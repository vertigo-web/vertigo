use std::rc::Rc;

use vertigo::{KeyDownEvent, RealDomId};
use web_sys::{Document, KeyboardEvent};

use crate::dom_driver_browser::DomDriverBrowser;
use crate::{dom_event::DomEvent};

fn find_event(
    dom_driver: &DomDriverBrowser,
    id: RealDomId,
) -> Option<Rc<dyn Fn(KeyDownEvent) -> bool>> {
    let all_nodes = dom_driver.find_all_nodes(id);

    for node_id in all_nodes {
        let on_input = dom_driver.get_from_node(
            &node_id,
            |elem| elem.on_input.clone()
        );
                                //cancel the bubbling event
        if on_input.is_some() {
            return None;
        }

        let on_key = dom_driver.get_from_node(
            &node_id,
            |elem| elem.on_keydown.clone()
        );

        if on_key.is_some() {
            return on_key;
        }
    }

    None
}

pub fn create_keydown_event(document: &Document, dom_driver: &DomDriverBrowser) -> DomEvent {
    let dom_driver = dom_driver.clone();

    DomEvent::new(document, "keydown", move |event: web_sys::Event| {
        let dom_id = dom_driver.find_dom_id(&event);

        let event_to_run = find_event(&dom_driver, dom_id);

        if let Some(event_to_run) = event_to_run {
            
            let event_keyboard = {
                use wasm_bindgen::JsCast;
                event.dyn_ref::<KeyboardEvent>().unwrap()
            };

            let event = KeyDownEvent {
                code: event_keyboard.code(),
                key: event_keyboard.key(),
                alt_key: event_keyboard.alt_key(),
                ctrl_key: event_keyboard.ctrl_key(),
                shift_key: event_keyboard.shift_key(),
                meta_key: event_keyboard.meta_key(),
            };

            let prevent_default = event_to_run(event);

            if prevent_default {
                event_keyboard.prevent_default();
                event_keyboard.stop_propagation();
            }
        }
    })
}