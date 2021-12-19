use crate::computed::Client;

use crate::virtualdom::models::{
    realdom_id::RealDomId, realdom_node::RealDomElement, vdom_component_id::VDomComponentId,
};

pub struct RealDomComponent {
    pub id: VDomComponentId, // for comparison
    pub subscription: Client,
    pub node: RealDomElement,
}

impl RealDomComponent {
    pub fn dom_id(&self) -> RealDomId {
        self.node.id_dom()
    }
}
