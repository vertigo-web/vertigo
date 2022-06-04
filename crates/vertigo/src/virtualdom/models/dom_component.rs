use std::any::Any;

use crate::virtualdom::models::{
    dom_id::DomId, dom_node::DomElement,
};

use super::vdom_component_id::VDomComponentId;

pub struct DomComponent {
    pub id: VDomComponentId,           // for comparison
    pub subscription: Box<dyn Any>,
    pub node: DomElement,
}

impl DomComponent {
    pub fn id_dom(&self) -> DomId {
        self.node.id_dom()
    }
}
