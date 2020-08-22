use crate::vdom::{
    models::{
        RealDom::{
            RealDomNodeId,
        },
    },
    DomDriver::DomDriver::DomDriverTrait,
};

pub struct DomDriverPrint {}

impl DomDriverPrint {
    pub fn new() -> DomDriverPrint {
        DomDriverPrint {}
    }
}

impl DomDriverTrait for DomDriverPrint {
    fn createNode(&self, id: RealDomNodeId, name: String) {
        log::info!("createNode {} - {}", id, name);
    }

    fn createText(&self, id: RealDomNodeId, value: String) {
        log::info!("createText {} {}", id, value);
    }

    fn setAttr(&self, id: RealDomNodeId, key: String, value: String) {
        log::info!("setAttr {} {} {}", id, key, value);
    }

    fn addChild(&self, idParent: RealDomNodeId, idPrev: Option<RealDomNodeId>, idChild: RealDomNodeId) {
        log::info!("addChild {} {:?} {}", idParent, idPrev, idChild);
    }

    fn remove(&self, id: RealDomNodeId) {
        log::info!("remove {}", id);
    }
}