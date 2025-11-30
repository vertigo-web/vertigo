use std::rc::Rc;

use crate::{
    computed::{struct_mut::ValueMut, DropResource},
    css::get_css_manager,
    driver_module::{get_driver_dom, StaticString},
    Css, DomId,
};

struct DomElementClassMergeInner {
    id_dom: DomId,
    css_name: Option<(Css, Option<String>)>,
    attr_name: Option<Rc<String>>,
    _suspense_drop: Option<DropResource>,
    suspense_css: Option<Css>,
    // Class name (autocss_X) and optional class_name_hint
    command_last_sent: Option<(String, Option<String>)>,
}

impl DomElementClassMergeInner {
    fn new(id_dom: DomId) -> Self {
        Self {
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
            let css = get_css_manager().get_class_name(css);
            result.push(css);
            class_name_hint = debug_class_name.clone();
        }

        if let Some(suspense_css) = &self.suspense_css {
            let suspense_css = get_css_manager().get_class_name(suspense_css);
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

            get_driver_dom().set_attr(self.id_dom, "class", &new_command);

            if let Some(class_name_hint) = class_name_hint {
                get_driver_dom().set_attr(self.id_dom, "v-css", &class_name_hint);
            }
        }
    }
}

#[derive(Clone)]
pub struct DomElementClassMerge {
    inner: Rc<ValueMut<DomElementClassMergeInner>>,
}

impl DomElementClassMerge {
    pub fn new(id_dom: DomId) -> Self {
        Self {
            inner: Rc::new(ValueMut::new(DomElementClassMergeInner::new(id_dom))),
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

        let id = self.inner.map(|state| state.id_dom);

        match value {
            Some(value) => {
                get_driver_dom().set_attr(id, name, &value);
            }
            None => {
                get_driver_dom().remove_attr(id, name);
            }
        }
    }
}
