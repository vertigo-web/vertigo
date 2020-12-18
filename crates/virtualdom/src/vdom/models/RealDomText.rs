use crate::{
    vdom::{
        models::{
            RealDomId::RealDomId,
        },
    },
    driver::DomDriver,
};

pub struct RealDomText {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    pub value: String,
}

impl RealDomText {
    pub fn new(domDriver: DomDriver, value: String) -> RealDomText {
        let id = RealDomId::default();

        domDriver.createText(id.clone(), &value);

        RealDomText {
            domDriver,
            idDom: id,
            value,
        }
    }

    pub fn update(&mut self, newValue: &str) {
        if self.value != *newValue {
            self.value = newValue.to_string();
        }
    }
}

impl Drop for RealDomText {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
