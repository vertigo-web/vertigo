use std::rc::Rc;

use vertigo::{KeyDownEvent, RealDomId, utils::BoxRefCell};
use web_sys::{Document, KeyboardEvent};

use crate::{DomDriverBrowserInner, dom_event::DomEvent, events::find_dom_id};

use super::{find_all_nodes, get_from_node};

fn find_event(
    inner: &Rc<BoxRefCell<DomDriverBrowserInner>>,
    id: RealDomId,
) -> Option<Rc<dyn Fn(KeyDownEvent) -> bool>> {
    let all_nodes = find_all_nodes(inner, id);

    for node_id in all_nodes {
        let on_input = get_from_node(
            inner,
            &node_id,
            |elem| elem.on_input.clone()
        );
                                //cancel the bubbling event
        if on_input.is_some() {
            return None;
        }

        let on_key = get_from_node(
            inner,
            &node_id,
            |elem| elem.on_keydown.clone()
        );

        if on_key.is_some() {
            return on_key;
        }
    }

    None
}

pub fn create_keydown_event(document: &Document, inner: &Rc<BoxRefCell<DomDriverBrowserInner>>) -> DomEvent {
    let inner = inner.clone();

    DomEvent::new(document, "keydown", move |event: web_sys::Event| {
        let dom_id = find_dom_id(&event);

        let event_to_run = find_event(&inner, dom_id);

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