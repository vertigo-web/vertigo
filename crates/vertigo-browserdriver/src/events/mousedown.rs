use std::rc::Rc;

use vertigo::{RealDomId};
use web_sys::{Element};

use crate::dom_driver_browser::{DomDriverBrowser};
use crate::dom_event::DomEvent;


fn find_event_click(
    dom_driver: &DomDriverBrowser,
    id: RealDomId,
) -> Option<Rc<dyn Fn()>> {

    let all_nodes = dom_driver.find_all_nodes(id);

    for node_id in all_nodes {

        let on_click = dom_driver.get_from_node(
            &node_id,
            |elem| elem.on_click.clone()
        );

        if on_click.is_some() {
            return on_click;
        }
    }

    None
}

pub fn create_mousedown_event(dom_driver: &DomDriverBrowser, root: &Element) -> DomEvent {
    let dom_driver = dom_driver.clone();

    DomEvent::new(&root, "mousedown", move |event: web_sys::Event| {
        let dom_id = dom_driver.find_dom_id(&event);

        let event_to_run = find_event_click(&dom_driver, dom_id);

        if let Some(event_to_run) = event_to_run {
            event_to_run();
        }
    })
}