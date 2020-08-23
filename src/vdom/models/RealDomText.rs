use crate::vdom::{
    models::{
        RealDomId::RealDomId,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};

pub struct RealDomText {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    value: String,
}

impl RealDomText {
    pub fn new(domDriver: DomDriver, value: String) -> RealDomText {
        let id = RealDomId::new();

        domDriver.createText(id.clone(), &value);

        let node = RealDomText {
            domDriver,
            idDom: id,
            value,
        };

        node
    }
}

impl Drop for RealDomText {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
