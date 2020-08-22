use crate::vdom::{
    models::{
        RealDomNodeId::RealDomNodeId,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};

pub struct RealDomComment {
    domDriver: DomDriver,
    pub idDom: RealDomNodeId,
    value: String,
}

impl RealDomComment {
    pub fn new(domDriver: DomDriver, value: String) -> RealDomComment {
        let id = RealDomNodeId::new();

        domDriver.createComment(id.clone(), &value);

        let node = RealDomComment {
            domDriver,
            idDom: id,
            value,
        };

        node
    }
}

impl Drop for RealDomComment {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
