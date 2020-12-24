use crate::{computed::BoxRefCell, driver::DomDriver, virtualdom::{
        models::{
            RealDomId::RealDomId,
        },
    }};

pub struct RealDomText {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    value: BoxRefCell<String>,
}

impl RealDomText {
    pub fn new(domDriver: DomDriver, value: String) -> RealDomText {
        let id = RealDomId::default();

        domDriver.create_text(id.clone(), &value);

        RealDomText {
            domDriver,
            idDom: id,
            value: BoxRefCell::new(value),
        }
    }

    pub fn update(&self, newValue: &str) {
        let should_update = self.value.change(newValue, |state, newValue| {
            if *state != *newValue {
                *state = newValue.to_string();
                true
            } else {
                false
            }
        });

        if should_update {
            self.domDriver.update_text(self.idDom.clone(), newValue);
        }
    }

    pub fn get_value(&self) -> String {
        self.value.get(|state| {
            (*state).clone()
        })
    }
}

impl Drop for RealDomText {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
