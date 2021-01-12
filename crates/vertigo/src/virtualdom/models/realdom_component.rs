use crate::computed::{
    Client,
};

use crate::virtualdom::{
    models::{
        realdom_node::RealDomElement,
        vdom_component_id::VDomComponentId,
        realdom_id::RealDomId,
    },
};

pub struct RealDomComponent {
    pub id: VDomComponentId,                    //do porównywania
    pub subscription: Client,                   //Subskrybcją, , wstawia do handler
    pub node: RealDomElement,
}

impl RealDomComponent {
    pub fn dom_id(&self) -> RealDomId {
        self.node.id_dom()
    }
}
