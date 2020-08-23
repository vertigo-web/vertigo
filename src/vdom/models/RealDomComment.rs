use crate::vdom::{
    models::{
        RealDomId::RealDomId,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};

pub struct RealDomComment {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    value: String,
}

impl RealDomComment {
    pub fn new(domDriver: DomDriver, value: String) -> RealDomComment {
        let id = RealDomId::new();

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
