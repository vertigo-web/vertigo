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
    pub value: String,
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

    pub fn update(&mut self, newValue: &String) {
        if self.value != *newValue {
            self.value = newValue.clone();
        }
    }
}

impl Drop for RealDomText {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
