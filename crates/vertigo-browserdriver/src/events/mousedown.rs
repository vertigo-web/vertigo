use std::rc::Rc;

use vertigo::utils::BoxRefCell;
use web_sys::{Element};

use crate::{DomDriverBrowserInner, dom_event::{DomEvent, DomEventDisconnect}};

use super::{find_dom_id, find_event};


pub fn create_mousedown_event(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, root: &Element) -> DomEventDisconnect {
    let inner = inner.clone();

    DomEvent::new_event(&root, "mousedown", move |event: web_sys::Event| {
        let dom_id = find_dom_id(&event);

        let event_to_run = find_event(&inner, dom_id, |item| &item.on_click);

        if let Some(event_to_run) = event_to_run {
            event_to_run();
        }
    })
}