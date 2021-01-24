use std::rc::Rc;
use std::cmp::PartialEq;

use crate::computed::{
    Dependencies,
    Computed,
    GraphId,
};
use crate::utils::BoxRefCell;

pub struct Value<T: 'static> {
    id: GraphId,
    value: Rc<BoxRefCell<Rc<T>>>,
    pub deps: Dependencies,
}

impl<T: PartialEq + 'static> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            id: self.id,
            value: self.value.clone(),
            deps: self.deps.clone(),
        }
    }
}

impl<T: PartialEq + 'static> Value<T> {
    pub fn new(deps: Dependencies, value: T) -> Value<T> {
        Value {
            id: GraphId::default(),
            value: Rc::new(BoxRefCell::new(Rc::new(value))),
            deps
        }
    }

    pub fn set_value(&self, value: T) {
        self.deps.transaction(|| {
            let value_has_change = self.value.change(value, |state, value| {
                let value_has_change = **state != value;
                *state = Rc::new(value);
                value_has_change
            });

            if value_has_change {
                self.deps.trigger_change(self.id);
            }
        });
    }

    pub fn get_value(&self) -> Rc<T> {
        self.deps.report_parent_in_stack(self.id);

        self.value.get(|state| {
            state.clone()
        })
    }

    pub fn to_computed(&self) -> Computed<T> {
        let self_clone = self.clone();

        Computed::new(self.deps.clone(), move || {
            self_clone.get_value()
        })
    }

    pub fn id(&self) -> GraphId {
        self.id
    }
}

impl<T: PartialEq + 'static> PartialEq for Value<T> {
    fn eq(&self, other: &Value<T>) -> bool {
        self.id == other.id
    }
}
