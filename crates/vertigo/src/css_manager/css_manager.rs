use std::collections::HashMap;
use std::rc::Rc;

use crate::utils::BoxRefCell;

use crate::driver::DomDriver;
use crate::virtualdom::models::css::{
    Css,
    CssGroup
};
use super::{get_selector::get_selector, next_id::NextId, transform_css::transform_css};

struct CssManagerInner {
    driver: DomDriver,
    next_id: NextId,
    ids_static: HashMap<&'static str, u64>,
    ids_dynamic: HashMap<String, u64>,
}

impl CssManagerInner {
    pub fn new(driver: DomDriver) -> CssManagerInner {
        CssManagerInner {
            driver,
            next_id: NextId::new(),
            ids_static: HashMap::new(),
            ids_dynamic: HashMap::new(),
        }
    }

    fn insert_css(&mut self, css: &str) -> u64 {
        let (class_id, css_selectors) = transform_css(css, &mut self.next_id);

        for (selector, selector_data) in css_selectors {
            self.driver.insert_css(&selector, &selector_data);
        }

        class_id
    }
}

#[derive(Clone)]
pub struct CssManager {
    inner:  Rc<BoxRefCell<CssManagerInner>>,
}

impl CssManager {
    pub fn new(driver: &DomDriver) -> CssManager {
        CssManager {
            inner: Rc::new(BoxRefCell::new(CssManagerInner::new(driver.clone()), "css manager"))
        }
    }

    fn get_static(&self, css: &'static str) -> String {
         self.inner.change(css, |state, css| {
            if let Some(class_id) = state.ids_static.get(&css) {
                return get_selector(class_id);
            }

            let class_id = state.insert_css(css);
            state.ids_static.insert(css, class_id);

            get_selector(&class_id)
        })
    }

    fn get_dynamic(&self, css: &str) -> String {
        self.inner.change(css, |state, css| {
            if let Some(class_id) = state.ids_dynamic.get(css) {
                return get_selector(class_id);
            }

            let class_id = state.insert_css(css);
            state.ids_dynamic.insert(css.to_string(), class_id);

            get_selector(&class_id)
        })
    }

    pub fn get_class_name(&self, css: &Css) -> String {
        let mut out: Vec<String> = Vec::new();

        for item in css.groups.iter() {
            match item {
                CssGroup::CssStatic { value } => {
                    out.push(self.get_static(value));
                },
                CssGroup::CssDynamic { value } => {
                    out.push(self.get_dynamic(value));
                }
            }
        }

        out.join(" ")
    }
}
