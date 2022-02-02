use crate::{
    VDomElement
};

pub fn assert_empty(el: &VDomElement, name: &str) {
    assert_eq!(el.name, name);
    assert_eq!(el.children.len(), 0);
}
