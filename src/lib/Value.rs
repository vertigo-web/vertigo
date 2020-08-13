use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Computed::Computed,
};
struct ValueInner<T: 'static> {
    value: Rc<T>
}

impl<T: 'static> ValueInner<T> {
    fn new(value: T) -> ValueInner<T> {
        ValueInner {
            value: Rc::new(value)
        }
    }
}

pub struct Value<T: Debug + 'static> {
    id: u64,
    refCell: BoxRefCell<ValueInner<T>>,
    deps: Rc<Dependencies>,
}

impl<T: Debug + 'static> Value<T> {
    pub fn new(deps: Rc<Dependencies>, value: T) -> Rc<Value<T>> {
        Rc::new(Value {
            id: get_unique_id(),
            refCell: BoxRefCell::new(ValueInner::new(value)),
            deps
        })
    }

    pub fn setValue(self: &Rc<Value<T>>, value: T) /* -> Vec<Rc<Client>> */ {                          //TODO - trzeba odebrać i wywołać
        self.refCell.change(value, |state, value| {
            println!("Value::setValue {:?}", value);
            state.value = Rc::new(value);
        });

        self.deps.triggerChange(self.id);
    }

    pub fn getValue(&self) -> Rc<T> {
        let value = self.refCell.get(|state| {
            state.value.clone()
        });

        value
    }

    pub fn toComputed(self: &Rc<Value<T>>) -> Rc<Computed<T>> {
        let selfClone = self.clone();

        let getValue = Box::new(move || {
            selfClone.getValue()
        });

        let computed = Computed::new(self.deps.clone(), getValue);

        self.deps.addRelation(self.id, computed.getComputedRefresh());

        computed
    }
}
