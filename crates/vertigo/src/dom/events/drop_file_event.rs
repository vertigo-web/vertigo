use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropFileItem {
    pub name: String,
    pub data: Rc<Vec<u8>>,
}

impl DropFileItem {
    pub fn new(name: String, data: Vec<u8>) -> DropFileItem {
        DropFileItem {
            name,
            data: Rc::new(data),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DropFileEvent {
    pub items: Vec<DropFileItem>,
}

impl DropFileEvent {
    pub fn new(items: Vec<DropFileItem>) -> DropFileEvent {
        DropFileEvent { items }
    }
}
