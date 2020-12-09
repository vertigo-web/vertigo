use std::collections::HashMap;
use std::rc::Rc;

use crate::computed::BoxRefCell::BoxRefCell;

use crate::vdom::driver::DomDriver::DomDriver;
use crate::vdom::models::Css::{
    Css,
    CssGroup
};

struct CssManagerInner {
    driver: DomDriver,
    counter: u64,
    idsStatic: HashMap<u64, u64>,
    idsDynamic: HashMap<String, u64>,
}

impl CssManagerInner {
    pub fn new(driver: DomDriver) -> CssManagerInner {
        CssManagerInner {
            driver,
            counter: 0,
            idsStatic: HashMap::new(),
            idsDynamic: HashMap::new(),
        }
    }

    pub fn getNextClassId(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}

pub struct CssManager {
    inner:  Rc<BoxRefCell<CssManagerInner>>,
}

impl Clone for CssManager {
    fn clone(&self) -> Self {
        CssManager {
            inner: self.inner.clone()
        }
    }
}

fn getSelector(id: u64) -> String {
    format!("autocss_{}", id)
}

impl CssManager {
    pub fn new(driver: &DomDriver) -> CssManager {
        CssManager {
            inner: Rc::new(BoxRefCell::new(CssManagerInner::new(driver.clone())))
        }
    }

    fn getStatic(&self, css: &'static str) -> String {
        let cssStaticId: u64 = css.as_ptr() as u64;

         self.inner.change((cssStaticId,css), |state, (cssStaticId, css)| {
            if let Some(classId) = state.idsStatic.get(&cssStaticId) {
                return getSelector(*classId);
            }

            let classId = state.getNextClassId();
            let selector = getSelector(classId);
            state.driver.insertCss(format!(".{}", selector), css.to_string());
            state.idsStatic.insert(cssStaticId, classId);

            selector
        })
    }

    fn getDynamic(&self, css: &str) -> String {
        self.inner.change(css, |state, css| {
            if let Some(classId) = state.idsDynamic.get(css) {
                return getSelector(*classId);
            }

            let classId = state.getNextClassId();
            let selector = getSelector(classId);
            state.driver.insertCss(format!(".{}", selector), css.to_string());
            state.idsDynamic.insert(css.to_string(), classId);

            selector
        })
    }

    pub fn getClassName(&self, css: &Css) -> String {
        let mut out: Vec<String> = Vec::new();

        for item in css.groups.iter() {
            match item {
                CssGroup::CssStatic { value } => {
                    out.push(self.getStatic(value));
                },
                CssGroup::CssDynamic { value } => {
                    out.push(self.getDynamic(value));
                }
            }
        }

        out.join(" ")
    }
}

