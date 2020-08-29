use std::collections::HashMap;

use crate::vdom::models::{
    VDom::VDom,
};

pub struct VDomNode {
    pub name: String,
    pub attr: HashMap<String, String>,
    pub child: Vec<VDom>,
    pub onClick: Option<Box<dyn Fn()>>,
}
