use std::rc::Rc;

use crate::{DomId, Driver, struct_mut::ValueMut, DropResource, Css, get_driver};

struct DomElementClassMergeInner {
    driver: Driver,
    id_dom: DomId,
    css_name: Option<Css>,
    attr_name: Option<String>,
    _suspense_drop: Option<DropResource>,
    suspense_css: Option<Css>,
    command_last_sent: Option<String>,
}

impl DomElementClassMergeInner {
    fn new(driver: Driver, id_dom: DomId) -> Self {
        Self {
            driver,
            id_dom,
            css_name: None,
            attr_name: None,
            _suspense_drop: None,
            suspense_css: None,
            command_last_sent: None,
        }
    }

    fn get_new_command(&self) -> Option<String> {
        let mut result = Vec::new();

        if let Some(attr) = &self.attr_name {
            result.push(attr.clone());
        }

        if let Some(css) = &self.css_name {
            let css = get_driver().inner.css_manager.get_class_name(css);
            result.push(css);
        }

        if let Some(suspense_css) = &self.suspense_css {
            let suspense_css = get_driver().inner.css_manager.get_class_name(suspense_css);
            result.push(suspense_css);
        }

        if result.is_empty() {
            None
        } else {
            Some(result.join(" "))
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

    pub fn remove_attribute(&self) {
        self.inner.change(|state| {
            state.attr_name = None;
            state.refresh_dom();
        });
    }

    pub fn set_css(&self, new_value: Css) {
        self.inner.change(|state| {
            state.css_name = Some(new_value);
            state.refresh_dom();
        });
    }

    pub fn set_suspense_attr(&self, callback: Option<fn(bool) -> Css>) {
        let new_callback = match callback {
            Some(callback) => {
                let self_clone = self.clone();

                Some(move |is_loading: bool| {
                    let css = callback(is_loading);

                    self_clone.inner.change(|state| {
                        state.suspense_css = Some(css);
                        state.refresh_dom();
                    });
                })
            },
            None => None,
        };

        self.inner.change(|state| {

            let driver = get_driver();

            let drop = match new_callback {
                Some(callback) => {
                    Some(driver.inner.dom.dom_suspense.set_layer_callback(state.id_dom, callback))
                }
                None => {
                    None
                }
            };

            state._suspense_drop = drop;
        });
    }
}
