use crate::GraphId;
use crate::computed::Client;

use crate::virtualdom::models::{
    realdom_id::RealDomId, realdom_node::RealDomElement,
};

pub struct RealDomComponent {
    pub id: GraphId,           // for comparison
    pub subscription: Client,
    pub node: RealDomElement,
}

impl RealDomComponent {
    pub fn dom_id(&self) -> RealDomId {
        self.node.id_dom()
    }
}
