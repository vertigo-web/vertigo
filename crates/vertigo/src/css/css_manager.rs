use std::rc::Rc;

use crate::struct_mut::{HashMapMut, InnerValue};

use super::{
    css_structs::{Css, CssGroup},
    get_selector::get_selector,
    next_id::NextId,
    transform_css::transform_css,
};

type InsertFn = dyn Fn(Option<String>, String);

struct CssManagerInner {
    insert_css: Box<InsertFn>,
    next_id: NextId,
    ids_static: HashMapMut<&'static str, u64>,
    ids_dynamic: HashMapMut<String, u64>,
    bundle: InnerValue<String>,
}

impl CssManagerInner {
    pub fn new(insert_css: Box<InsertFn>) -> CssManagerInner {
        CssManagerInner {
            insert_css,
            next_id: NextId::new(),
            ids_static: HashMapMut::new(),
            ids_dynamic: HashMapMut::new(),
            bundle: InnerValue::new("".to_string()),
        }
    }

    fn insert_css(&self, css: &str) -> u64 {
        let (class_id, css_selectors) = transform_css(css, &self.next_id);

        for (selector, selector_data) in css_selectors {
            (self.insert_css)(Some(selector), selector_data);
        }

        class_id
    }

    pub fn register_bundle(&self, bundle: String) {
        *self.bundle.get_mut() = bundle.clone();
        (self.insert_css)(None, bundle)
    }
}

#[derive(Clone)]
pub struct CssManager {
    inner: Rc<CssManagerInner>,
}

impl CssManager {
    /// Create CssManager with css callback
    ///
    /// Callback accepts selector -> styles for autocss styling and None -> styles for bundles (i.e. a tailwind bundle)
    pub fn new(insert_css: impl Fn(Option<String>, String) + 'static) -> CssManager {
        CssManager {
            inner: Rc::new(CssManagerInner::new(Box::new(insert_css))),
        }
    }

    fn get_static(&self, css: &'static str) -> String {
        if let Some(class_id) = self.inner.ids_static.get(&css) {
            return get_selector(&class_id);
        }

        let class_id = self.inner.insert_css(css);
        self.inner.ids_static.insert(css, class_id);

        get_selector(&class_id)
    }

    fn get_dynamic(&self, css: impl Into<String>) -> String {
        let css = css.into();
        if let Some(class_id) = self.inner.ids_dynamic.get(&css) {
            return get_selector(&class_id);
        }

        let class_id = self.inner.insert_css(css.as_str());
        self.inner.ids_dynamic.insert(css, class_id);

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

    pub fn register_bundle(&self, bundle: String) {
        self.inner.register_bundle(bundle);
    }
}
