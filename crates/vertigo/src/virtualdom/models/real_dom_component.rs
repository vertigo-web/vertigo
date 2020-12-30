use crate::computed::{
    Client,
};

use crate::virtualdom::{
    models::{
        real_dom_node::RealDomNode,
        v_dom_component_id::VDomComponentId,
        real_dom_id::RealDomId,
    },
};

pub struct RealDomComponent {
    pub id: VDomComponentId,                    //do porównywania
    pub subscription: Client,                   //Subskrybcją, , wstawia do handler
    pub node: RealDomNode,
}

impl RealDomComponent {
    pub fn domId(&self) -> RealDomId {
        self.node.idDom()
    }
}
