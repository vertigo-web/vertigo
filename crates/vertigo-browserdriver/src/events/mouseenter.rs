use std::rc::Rc;
use std::collections::HashMap;
use vertigo::{RealDomId, utils::BoxRefCell};
use web_sys::{Element};

use crate::{DomDriverBrowserInner, dom_event::{DomEvent, DomEventDisconnect}, transaction::Transaction};

use super::{find_dom_id, find_all_nodes};

struct VisitedNode {
    on_mouse_leave: Option<Rc<dyn Fn()>>,
}

impl VisitedNode {
    pub fn new(on_mouse_leave: Option<Rc<dyn Fn()>>) -> VisitedNode {
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
    inner: Rc<BoxRefCell<DomDriverBrowserInner>>,
    transaction: Transaction,
    nodes: BoxRefCell<HashMap::<RealDomId, VisitedNode>>,
}

impl VisitedNodeManager {
    fn new(inner: Rc<BoxRefCell<DomDriverBrowserInner>>, transaction: Transaction) -> VisitedNodeManager {
        let nodes = HashMap::new();

        VisitedNodeManager {
            inner,
            transaction,
            nodes: BoxRefCell::new(nodes, "VisitedNodeManager nodes")
        }
    }

    fn push_new_nodes(&self, new_nodes: Vec<RealDomId>) {
        let VisitedNodeManager {inner, transaction, nodes} = self;

        transaction.exec(move || {
            nodes.change((new_nodes, inner), |state, (new_nodes, inner)| {
                let mut new_state = HashMap::<RealDomId, VisitedNode>::new();

                for node_id in new_nodes {
                    let old_node = state.remove(&node_id);

                    if let Some(old_node) = old_node {
                        new_state.insert(node_id, old_node);
                        continue;
                    }

                    let (on_enter, on_leave) = inner.get_with_context(&node_id, |state, node_id| {
                        let item = state.elements.get(node_id);

                        if let Some(item) = item {
                            let on_enter = item.on_mouse_enter.clone();
                            let on_leave = item.on_mouse_leave.clone();

                            (on_enter, on_leave)
                        } else {
                            (None, None)
                        }
                    });

                    if let Some(on_enter) = on_enter {
                        on_enter();
                    }

                    new_state.insert(node_id, VisitedNode::new(on_leave));
                }

                std::mem::replace(state, new_state)
            });
        });
    }
}

pub fn create_mouseenter_event(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, root: &Element, transaction: Transaction) -> DomEventDisconnect {

    let inner = inner.clone();
    let current_visited = VisitedNodeManager::new(inner.clone(), transaction);

    DomEvent::new_event(&root, "mouseover",move |event: web_sys::Event| {
        let dom_id = find_dom_id(&event);
        let nodes = find_all_nodes(&inner, dom_id.clone());

        current_visited.push_new_nodes(nodes);
    })
}