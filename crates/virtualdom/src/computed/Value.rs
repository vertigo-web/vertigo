use std::rc::Rc;
use std::fmt::Debug;

use crate::computed::{
    BoxRefCell::BoxRefCell,
    Dependencies::Dependencies,
    Computed::Computed,
    GraphId::GraphId,
};

pub struct Value<T: Debug + 'static> {
    id: GraphId,
    value: Rc<BoxRefCell<Rc<T>>>,
    deps: Dependencies,
}

impl<T: Debug + 'static> Value<T> {
    pub fn new(deps: Dependencies, value: T) -> Value<T> {
        Value {
            id: GraphId::new(),
            value: Rc::new(BoxRefCell::new(Rc::new(value))),
            deps
        }
    }

    pub fn setValue(&self, value: T) {
        self.value.change(value, |state, value| {
            *state = Rc::new(value);
        });

        self.deps.triggerChange(self.id.clone());
    }

    pub fn getValue(&self) -> Rc<T> {
        self.deps.reportDependenceInStack(self.id.clone());

        let value = self.value.get(|state| {
            state.clone()
        });

        value
    }

    pub fn toComputed(&self) -> Computed<T> {
        let value = self.value.clone();
        let deps = self.deps.clone();

        let selfId = self.id.clone();

        Computed::new(deps.clone(), move || {
            deps.reportDependenceInStack(selfId.clone());
            value.get(|state| {
                state.clone()
            })
        })
    }
}
