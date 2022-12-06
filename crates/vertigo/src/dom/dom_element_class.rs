use std::rc::Rc;

use crate::{DomId, Driver, struct_mut::ValueMut};

struct DomElementClassMergeInner {
    driver: Driver,
    id_dom: DomId,
    css_name: Option<String>,
    attr_name: Option<String>,
    command_last_sent: Option<String>,
}

impl DomElementClassMergeInner {
    fn new(driver: Driver, id_dom: DomId) -> Self {
        Self {
            driver,
            id_dom,
            css_name: None,
            attr_name: None,
            command_last_sent: None,
        }
    }

    fn get_new_command(&self) -> Option<String> {
        match (&self.attr_name, &self.css_name) {
            (None, None) => None,
            (Some(attr), None) => Some(attr.clone()),
            (None, Some(css)) => Some(css.clone()),
            (Some(attr), Some(css)) => {
                Some([attr, " ", css].join(""))
            }
        }
    }

    fn refresh_dom(&mut self) {
        let new_command = self.get_new_command();

        if self.command_last_sent != new_command {
            self.command_last_sent = new_command.clone();

            let new_command = match new_command {
                Some(new_command) => new_command,
                None => "".to_string()
            };

            self.driver.inner.dom.set_attr(self.id_dom, "class", &new_command);
        }
    }
}

#[derive(Clone)]
pub struct DomElementClassMerge {
    inner: Rc<ValueMut<DomElementClassMergeInner>>,
}

impl DomElementClassMerge {
    pub fn new(driver: Driver, id_dom: DomId) -> Self {
        Self {
            inner: Rc::new(ValueMut::new(DomElementClassMergeInner::new(driver, id_dom)))
        }
    }

    pub fn set_attribute(&self, new_value: String) {
        self.inner.change(|state| {
            state.attr_name = Some(new_value);
            state.refresh_dom();
        });
    }

    pub fn set_css(&self, new_value: String) {
        self.inner.change(|state| {
            state.css_name = Some(new_value);
            state.refresh_dom();
        });
    }
}
