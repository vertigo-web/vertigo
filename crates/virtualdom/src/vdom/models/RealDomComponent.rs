use crate::computed::{
    Client::Client,
};

use crate::vdom::{
    models::{
        RealDomNode::RealDomNode,
        VDomComponentId::VDomComponentId,
        RealDomId::RealDomId,
    },
};

pub struct RealDomComponent {
    pub id: VDomComponentId,                    //do porównywania
    pub subscription: Client,                   //Subskrybcją, , wstawia do handler
    pub node: RealDomNode,
}

impl RealDomComponent {
    pub fn id(&self) -> RealDomId {
        self.node.idDom.clone()
    }
}
