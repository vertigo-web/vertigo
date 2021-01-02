use core::{
    cmp::PartialEq,
};
use alloc::{
    rc::Rc,
};

use crate::computed::{
    Dependencies,
    Computed,
    GraphId,
};
use crate::utils::BoxRefCell;

pub struct Value<T: 'static> {
    id: GraphId,
    value: Rc<BoxRefCell<Rc<T>>>,
    deps: Dependencies,
}

impl<T: PartialEq + 'static> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            id: self.id.clone(),
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

    pub fn new_value_wrap_width_computed(deps: Dependencies, value: T) -> Computed<Value<T>> {
        let value = deps.new_value(value);
        //let computed = value.to_computed();                   //TODO - uncommenting this line causes performance to down
        deps.new_computed_from(value)
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
        self.deps.report_parent_in_stack(self.id.clone());

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

    fn ne(&self, other: &Value<T>) -> bool {
        self.id != other.id
    }
}
