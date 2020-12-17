use crate::computed::{
    Client,
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
    pub fn domId(&self) -> RealDomId {
        self.node.idDom()
    }
}
