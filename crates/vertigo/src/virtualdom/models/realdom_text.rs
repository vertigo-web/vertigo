use crate::{
    driver::Driver,
    utils::BoxRefCell,
    virtualdom::models::realdom_id::RealDomId
};

pub struct RealDomText {
    dom_driver: Driver,
    id_dom: RealDomId,
    value: BoxRefCell<String>,
}

impl RealDomText {
    pub fn new(dom_driver: Driver, value: String) -> RealDomText {
        let id = RealDomId::default();

        dom_driver.create_text(id, &value);

        RealDomText {
            dom_driver,
            id_dom: id,
            value: BoxRefCell::new(value, "RealDomText"),
        }
    }

    pub fn update(&self, new_value: &str) {
        let should_update = self.value.change(new_value, |state, new_value| {
            if *state != *new_value {
                *state = new_value.to_string();
                true
            } else {
                false
            }
        });

        if should_update {
            self.dom_driver.update_text(self.id_dom, new_value);
        }
    }

    pub fn get_value(&self) -> String {
        self.value.get(|state| {
            (*state).clone()
        })
    }

    pub fn id_dom(&self) -> RealDomId {
        self.id_dom
    }
}

impl Drop for RealDomText {
    fn drop(&mut self) {
        self.dom_driver.remove_text(self.id_dom);
    }
}
