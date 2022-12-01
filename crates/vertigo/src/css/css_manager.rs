use std::{
    rc::Rc,
};

use crate::{
    dom::css::{Css, CssGroup},
    struct_mut::HashMapMut,
};

use super::{
    get_selector::get_selector,
    next_id::NextId,
    transform_css::transform_css,
};

type InsertFn = dyn Fn(&str, &str);

struct CssManagerInner {
    insert_css: Box<InsertFn>,
    next_id: NextId,
    ids_static: HashMapMut<&'static str, u64>,
    ids_dynamic: HashMapMut<String, u64>,
}

impl CssManagerInner {
    pub fn new(insert_css: Box<InsertFn>) -> CssManagerInner {
        CssManagerInner {
            insert_css,
            next_id: NextId::new(),
            ids_static: HashMapMut::new(),
            ids_dynamic: HashMapMut::new(),
        }
    }

    fn insert_css(&self, css: &str) -> u64 {
        let (class_id, css_selectors) = transform_css(css, &self.next_id);

        for (selector, selector_data) in css_selectors {
            (self.insert_css)(&selector, &selector_data);
        }

        class_id
    }
}

#[derive(Clone)]
pub struct CssManager {
    inner: Rc<CssManagerInner>,
}

impl CssManager {
    pub fn new(insert_css: impl Fn(&str, &str) + 'static) -> CssManager {
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
}
