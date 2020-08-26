use std::rc::Rc;

use virtualdom::{
    computed::{
        Dependencies::Dependencies,
        Value::Value,
    }
};

pub struct AppState {
    pub value: Value<u32>,
    pub at: Value<u32>
}

impl AppState {
    pub fn new(root: &Dependencies) -> Rc<AppState> {
        Rc::new(AppState {
            value: root.newValue(33),
            at: root.newValue(999),
        })
    }
}
