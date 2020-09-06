use std::collections::HashMap;
use std::rc::Rc;

use crate::computed::BoxRefCell::BoxRefCell;

use crate::vdom::models::Css::{
    Css,
    CssGroup
};

struct CssManagerInner {
    counter: u64,
    idsStatic: HashMap<u64, u64>,
    idsDynamic: HashMap<String, u64>,
}

impl CssManagerInner {
    pub fn new() -> CssManagerInner {
        CssManagerInner {
            counter: 0,
            idsStatic: HashMap::new(),
            idsDynamic: HashMap::new(),
        }
    }

    pub fn getNextId(&mut self) -> u64 {
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

impl CssManager {
    pub fn new() -> CssManager {
        CssManager {
            inner: Rc::new(BoxRefCell::new(CssManagerInner::new()))
        }
    }

    fn getCssId(&self, id: u64) -> String {
        format!("autocss_{}", id)
    }

    fn getStatic(&self, css: &'static str) -> String {
        let id: u64 = css.as_ptr() as u64;

        let idCss = self.inner.change(id, |state, id| {
            if let Some(idCss) = state.idsStatic.get(&id) {
                return *idCss;
            }

            let nextId = state.getNextId();

            state.idsStatic.insert(id, nextId);
            
            nextId
        });

        self.getCssId(idCss)
    }

    fn getDynamic(&self, css: &String) -> String {
        todo!();
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

