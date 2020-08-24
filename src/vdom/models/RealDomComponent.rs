use crate::lib::{
    Client::Client,
};

use crate::vdom::{
    models::{
        RealDomChild::RealDomChild,
        VDomComponentId::VDomComponentId,
        RealDomId::RealDomId,
    },
};

pub struct RealDomComponent {
    pub id: VDomComponentId,                    //do porównywania
    subscription: Client,                   //Subskrybcją, , wstawia do handler
    child: RealDomChild,
}

impl RealDomComponent {
    pub fn firstChildId(&self) -> RealDomId {
        self.child.firstChildId()
    }
    pub fn lastChildId(&self) -> RealDomId {
        self.child.lastChildId()
    }

    pub fn childIds(&self) -> Vec<RealDomId> {
        self.child.childIds()
    }
}
