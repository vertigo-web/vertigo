use crate::{utils::BoxRefCell, driver::DomDriver, virtualdom::{
        models::{
            realdom_id::RealDomId,
        },
    }};

pub struct RealDomText {
    dom_driver: DomDriver,
    pub id_dom: RealDomId,
    value: BoxRefCell<String>,
}

impl RealDomText {
    pub fn new(dom_driver: DomDriver, value: String) -> RealDomText {
        let id = RealDomId::default();

        dom_driver.create_text(id.clone(), &value);

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
            self.dom_driver.update_text(self.id_dom.clone(), new_value);
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
        self.dom_driver.remove(self.id_dom.clone());
    }
}
