use std::collections::HashMap;

use crate::vdom::models::{
    VDom::VDom,
};

#[derive(Clone)]
pub struct VDomNode {
    pub name: String,
    pub attr: HashMap<String, String>,
    pub child: Vec<VDom>,
}
