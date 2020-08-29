use std::collections::HashMap;
use std::rc::Rc;

use crate::vdom::models::{
    VDom::VDom,
};

pub struct VDomNode {
    pub name: String,
    pub attr: HashMap<String, String>,
    pub child: Vec<VDom>,
    pub onClick: Option<Rc<dyn Fn()>>,
}
