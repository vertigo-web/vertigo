use std::rc::Rc;

use crate::{
    dom::callback::SuspenseCallback, driver_module::StaticString, get_driver, struct_mut::ValueMut,
    Css, DomId, Driver, DropResource,
};

struct DomElementClassMergeInner {
    driver: Driver,
    id_dom: DomId,
    css_name: Option<(Css, Option<String>)>,
    attr_name: Option<Rc<String>>,
    _suspense_drop: Option<DropResource>,
    suspense_css: Option<Css>,
    // Class name (autocss_X) and optional class_name_hint
    command_last_sent: Option<(String, Option<String>)>,
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

    fn get_new_command(&self) -> Option<(String, Option<String>)> {
        let mut result = Vec::new();
        let mut class_name_hint = None;

        if let Some(attr) = &self.attr_name {
            result.push(attr.to_string());
        }

        if let Some((css, debug_class_name)) = &self.css_name {
            let css = get_driver().inner.css_manager.get_class_name(css);
            result.push(css);
            class_name_hint = debug_class_name.clone();
        }

        if let Some(suspense_css) = &self.suspense_css {
            let suspense_css = get_driver().inner.css_manager.get_class_name(suspense_css);
            result.push(suspense_css);
        }

        if result.is_empty() {
            None
        } else {
            Some((result.join(" "), class_name_hint))
        }
    }

    fn refresh_dom(&mut self) {
        let new_command = self.get_new_command();

        if self.command_last_sent != new_command {
            self.command_last_sent.clone_from(&new_command);

            let (new_command, class_name_hint) = match new_command {
                Some(new_command) => (new_command.0, new_command.1),
                None => ("".to_string(), None),
            };

            self.driver
                .inner
                .dom
                .set_attr(self.id_dom, "class", &new_command);

            if let Some(class_name_hint) = class_name_hint {
                self.driver
                    .inner
                    .dom
                    .set_attr(self.id_dom, "v-css", &class_name_hint);
            }
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
            inner: Rc::new(ValueMut::new(DomElementClassMergeInner::new(
                driver, id_dom,
            ))),
        }
    }

    pub fn set_attribute(&self, new_value: Rc<String>) {
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

    pub fn set_css(&self, new_value: Css, debug_class_name: Option<String>) {
        self.inner.change(|state| {
            state.css_name = Some((new_value, debug_class_name));
            state.refresh_dom();
        });
    }

    pub fn set_suspense_attr(&self, callback: Option<Rc<SuspenseCallback>>) {
        self.inner.change(|state| {
            let Some(callback) = callback else {
                state._suspense_drop = None;
                return;
            };

            let drop = get_driver()
                .inner
                .dom
                .dom_suspense
                .set_layer_callback(state.id_dom, {
                    let self_clone = self.clone();

                    move |is_loading: bool| {
                        let css = callback(is_loading);

                        self_clone.inner.change(|state| {
                            state.suspense_css = Some(css);
                            state.refresh_dom();
                        });
                    }
                });

            state._suspense_drop = Some(drop);
        });
    }

    pub fn set_attr_value(&self, name: StaticString, value: Option<Rc<String>>) {
        if name.as_str() == "class" {
            match value {
                Some(value) => {
                    self.set_attribute(value);
                }
                None => {
                    self.remove_attribute();
                }
            }
            return;
        }

        let driver = get_driver();
        let id = self.inner.map(|state| state.id_dom);

        match value {
            Some(value) => {
                driver.inner.dom.set_attr(id, name, &value);
            }
            None => {
                driver.inner.dom.remove_attr(id, name);
            }
        }
    }
}
