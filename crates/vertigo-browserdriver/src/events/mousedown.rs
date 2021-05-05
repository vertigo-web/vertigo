use std::rc::Rc;

use vertigo::{RealDomId, utils::BoxRefCell};
use web_sys::{Element};

use crate::{DomDriverBrowserInner, dom_event::DomEvent};

use super::{find_all_nodes, find_dom_id, get_from_node};


fn find_event_click(
    inner: &Rc<BoxRefCell<DomDriverBrowserInner>>,
    id: RealDomId,
) -> Option<Rc<dyn Fn()>> {

    let all_nodes = find_all_nodes(inner, id);

    for node_id in all_nodes {

        let on_click = get_from_node(
            inner,
            &node_id,
            |elem| elem.on_click.clone()
        );

        if on_click.is_some() {
            return on_click;
        }
    }

    None
}

pub fn create_mousedown_event(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, root: &Element) -> DomEvent {
    let inner = inner.clone();

    DomEvent::new(&root, "mousedown", move |event: web_sys::Event| {
        let dom_id = find_dom_id(&event);

        let event_to_run = find_event_click(&inner, dom_id);

        if let Some(event_to_run) = event_to_run {
            event_to_run();
        }
    })
}