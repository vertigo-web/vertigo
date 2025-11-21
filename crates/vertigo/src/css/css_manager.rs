use std::rc::Rc;

use vertigo_macro::store;

use crate::{driver_module::get_driver_dom, struct_mut::{HashMapMut, InnerValue}};

use super::{
    css_structs::{Css, CssGroup},
    get_selector::get_selector,
    next_id::NextId,
    transform_css::transform_css,
};

#[store]
pub fn get_css_manager() -> Rc<CssManager> {
    Rc::new(CssManager::new())
}
pub struct CssManager {
    next_id: NextId,
    ids_static: HashMapMut<&'static str, u64>,
    ids_dynamic: HashMapMut<String, u64>,
    bundle: InnerValue<String>,
}

impl CssManager {
    /// Create CssManager with css callback
    ///
    /// Callback accepts selector -> styles for autocss styling and None -> styles for bundles (i.e. a tailwind bundle)
    fn new() -> CssManager {
        CssManager {
            next_id: NextId::new(),
            ids_static: HashMapMut::new(),
            ids_dynamic: HashMapMut::new(),
            bundle: InnerValue::new("".to_string()),
        }
    }

    fn insert_css(&self, css: &str) -> u64 {
        let (class_id, css_selectors) = transform_css(css, &self.next_id);

        let dom = get_driver_dom();

        for (selector, selector_data) in css_selectors {
            dom.insert_css(Some(selector), selector_data)
        }

        class_id
    }

    pub fn register_bundle(&self, bundle: String) {
        *self.bundle.get_mut() = bundle.clone();
        get_driver_dom().insert_css(None, bundle)
    }

    fn get_static(&self, css: &'static str) -> String {
        if let Some(class_id) = self.ids_static.get(&css) {
            return get_selector(&class_id);
        }

        let class_id = self.insert_css(css);
        self.ids_static.insert(css, class_id);

        get_selector(&class_id)
    }

    fn get_dynamic(&self, css: impl Into<String>) -> String {
        let css = css.into();
        if let Some(class_id) = self.ids_dynamic.get(&css) {
            return get_selector(&class_id);
        }

        let class_id = self.insert_css(css.as_str());
        self.ids_dynamic.insert(css, class_id);

        get_selector(&class_id)
    }

    pub fn get_class_name(&self, css: &Css) -> String {
        let mut out: Vec<String> = Vec::new();

        for item in css.groups.iter() {
            match item {
                CssGroup::CssStatic { value } => {
                    out.push(self.get_static(value));
                }
                CssGroup::CssDynamic { value } => {
                    out.push(self.get_dynamic(value));
                }
                CssGroup::CssMedia { query, rules } => {
                    out.push("@media".to_string());
                    out.push(query.clone());
                    for rule in rules {
                        out.push(self.get_dynamic(rule));
                    }
                }
            }
        }

        out.join(" ")
    }
}
