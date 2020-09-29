use crate::computed::{
    Client::Client,
};

use crate::vdom::{
    models::{
        RealDomChildList::RealDomChildList,
        VDomComponentId::VDomComponentId,
        RealDomId::RealDomId,
    },
};

pub struct RealDomComponent {
    pub id: VDomComponentId,                    //do porównywania
    pub subscription: Client,                   //Subskrybcją, , wstawia do handler
    pub child: RealDomChildList,
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
