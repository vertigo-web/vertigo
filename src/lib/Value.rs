use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Computed::Computed,
};

pub struct Value<T: Debug + 'static> {
    id: u64,
    value: Rc<BoxRefCell<Rc<T>>>,
    deps: Dependencies,
}

impl<T: Debug + 'static> Value<T> {
    pub fn new(deps: Dependencies, value: T) -> Value<T> {
        Value {
            id: get_unique_id(),
            value: Rc::new(BoxRefCell::new(Rc::new(value))),
            deps
        }
    }

    pub fn setValue(&self, value: T) {
        self.value.change(value, |state, value| {
            *state = Rc::new(value);
        });

        self.deps.triggerChange(self.id);
    }

    // pub fn getValue(&self) -> Rc<T> {
    //     let value = self.value.get(|state| {
    //         state.clone()
    //     });

    //     value
    // }

    pub fn toComputed(&self) -> Computed<T> {

        let value = self.value.clone();
        let deps = self.deps.clone();

        let selfId = self.id;

        Computed::new(deps.clone(), move || {
            deps.reportDependenceInStack(selfId);
            value.get(|state| {
                state.clone()
            })
        })
    }
}
