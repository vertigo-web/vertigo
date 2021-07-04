use std::rc::Rc;
use std::collections::HashMap;
use vertigo::{RealDomId, computed::Dependencies, utils::BoxRefCell};
use web_sys::{Element};

use crate::dom_driver_browser::DomDriverBrowser;
use crate::dom_event::DomEvent;

struct VisitedNode {
    on_mouse_leave: Option<Rc<dyn Fn()>>,
}

impl VisitedNode {
    pub fn new(on_mouse_enter: Option<Rc<dyn Fn()>>, on_mouse_leave: Option<Rc<dyn Fn()>>) -> VisitedNode {

        if let Some(on_mouse_enter) = on_mouse_enter {
            on_mouse_enter();
        }

        VisitedNode {
            on_mouse_leave,
        }
    }
}
impl Drop for VisitedNode {
    fn drop(&mut self) {
        let on_mouse_leave = std::mem::replace(&mut self.on_mouse_leave, None);

        if let Some(on_mouse_leave) = on_mouse_leave {
            on_mouse_leave();
        }
    }
}

//struktura do zarządzania ostnio odwiedzonymi węzłami

struct VisitedNodeManager {
    dom_driver: DomDriverBrowser,
    dependencies: Dependencies,
    nodes: BoxRefCell<HashMap::<RealDomId, VisitedNode>>,
}

impl VisitedNodeManager {
    fn new(dom_driver: &DomDriverBrowser, dependencies: &Dependencies) -> VisitedNodeManager {
        let nodes = HashMap::new();

        VisitedNodeManager {
            dom_driver: dom_driver.clone(),
            dependencies: dependencies.clone(),
            nodes: BoxRefCell::new(nodes, "VisitedNodeManager nodes")
        }
    }

    fn push_new_nodes(&self, new_nodes: Vec<RealDomId>) {
        let VisitedNodeManager {dom_driver, dependencies, nodes} = self;

        dependencies.transaction(move || {
            nodes.change((new_nodes, dom_driver), |state, (new_nodes, dom_driver)| {
                let mut new_state = HashMap::<RealDomId, VisitedNode>::new();

                for node_id in new_nodes {
                    let old_node = state.remove(&node_id);

                    if let Some(old_node) = old_node {
                        new_state.insert(node_id, old_node);
                        continue;
                    }

                    let on_enter = dom_driver.get_from_node(
                        &node_id,
                        |elem| elem.on_mouse_enter.clone()
                    );

                    let on_leave = dom_driver.get_from_node(
                        &node_id,
                        |elem| elem.on_mouse_leave.clone()
                    );

                    new_state.insert(node_id, VisitedNode::new(on_enter, on_leave));
                }

                std::mem::replace(state, new_state)
            });
        });
    }
}

pub fn create_mouseenter_event(dom_driver: &DomDriverBrowser, root: &Element, dependencies: &Dependencies) -> DomEvent {

    let dom_driver = dom_driver.clone();
    let current_visited = VisitedNodeManager::new(&dom_driver, dependencies);

    DomEvent::new(&root, "mouseover",move |event: web_sys::Event| {
        let dom_id = dom_driver.find_dom_id(&event);
        let nodes = dom_driver.find_all_nodes(dom_id.clone());

        current_visited.push_new_nodes(nodes);
    })
}