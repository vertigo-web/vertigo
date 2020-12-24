use std::rc::Rc;

use crate::computed::{
    BoxRefCell,
    Dependencies,
    Computed,
    GraphId,
};

#[derive(Clone)]
pub struct Value<T: 'static> {
    id: GraphId,
    value: Rc<BoxRefCell<Rc<T>>>,
    deps: Dependencies,
}

impl<T: 'static> Value<T> {
    pub fn new(deps: Dependencies, value: T) -> Value<T> {
        Value {
            id: GraphId::default(),
            value: Rc::new(BoxRefCell::new(Rc::new(value))),
            deps
        }
    }

    pub fn set_value(&self, value: T) {
        self.deps.transaction(|| {
            self.value.change(value, |state, value| {
                *state = Rc::new(value);
            });

            self.deps.trigger_change(self.id.clone());
        });
    }

    pub fn get_value(&self) -> Rc<T> {
        self.deps.report_dependence_in_stack(self.id.clone());

        self.value.get(|state| {
            state.clone()
        })
    }

    pub fn to_computed(&self) -> Computed<T> {
        let value = self.value.clone();
        let deps = self.deps.clone();

        let self_id = self.id.clone();

        Computed::new(deps.clone(), move || {
            deps.report_dependence_in_stack(self_id.clone());
            value.get(|state| {
                state.clone()
            })
        })
    }
}
