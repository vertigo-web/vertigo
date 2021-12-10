use std::rc::Rc;
use std::collections::HashMap;

use vertigo::{RealDomId, Dependencies, utils::BoxRefCell};

use super::driver_data::DriverData;

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

pub(crate) struct VisitedNodeManager {
    driver_data: Rc<DriverData>,
    dependencies: Dependencies,
    nodes: BoxRefCell<HashMap::<RealDomId, VisitedNode>>,
}

impl VisitedNodeManager {
    pub(crate) fn new(driver_data: &Rc<DriverData>, dependencies: &Dependencies) -> VisitedNodeManager {
        let nodes = HashMap::new();

        VisitedNodeManager {
            driver_data: driver_data.clone(),
            dependencies: dependencies.clone(),
            nodes: BoxRefCell::new(nodes, "VisitedNodeManager nodes")
        }
    }

    pub fn clear(&self) {
        let VisitedNodeManager {dependencies, nodes, ..} = self;

        dependencies.transaction(move || {
            nodes.change((), |state, _| {
                let new_state = HashMap::<RealDomId, VisitedNode>::new();
                let _ = std::mem::replace(state, new_state);
            })
        });
    }

    pub fn push_new_nodes(&self, new_nodes: Vec<RealDomId>) {
        let VisitedNodeManager {driver_data, dependencies, nodes} = self;

        dependencies.transaction(move || {
            nodes.change((new_nodes, driver_data), |state, (new_nodes, driver_data)| {
                let mut new_state = HashMap::<RealDomId, VisitedNode>::new();

                for node_id in new_nodes {
                    let old_node = state.remove(&node_id);

                    if let Some(old_node) = old_node {
                        new_state.insert(node_id, old_node);
                        continue;
                    }

                    let on_enter = driver_data.get_from_node(
                        &node_id,
                        |elem| elem.on_mouse_enter.clone()
                    );

                    let on_leave = driver_data.get_from_node(
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
