use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Computed::{
        Computed,
        ComputedBuilder,
    },
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

    pub fn setValue(&self, value: T) /* -> Vec<Rc<Client>> */ {                          //TODO - trzeba odebrać i wywołać
        self.value.change(value, |state, value| {
            println!("Value::setValue {:?}", value);
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
        let builder = ComputedBuilder::new(deps);
        let refresh = builder.getComputedRefresh();

        let getValue = Box::new(move || {
            value.get(|state| {
                state.clone()
            })
        });

        let computed = builder.build(getValue);

        self.deps.addRelation(self.id, refresh);

        computed
    }
}
