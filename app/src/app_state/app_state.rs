use virtualdom::{
    computed::{
        Dependencies::Dependencies,
        Value::Value,
        Computed::Computed,
    }
};

use crate::simple_counter::simple_counter::SimpleCounter;
use virtualdom::vdom::StateBox::StateBox;

pub struct AppState {
    pub value: Value<u32>,
    pub at: Value<u32>,
    pub counter1: StateBox<SimpleCounter>,
    pub counter2: StateBox<SimpleCounter>,
    pub counter3: StateBox<SimpleCounter>,

    pub suma: Computed<u32>,
}

impl AppState {
    pub fn new(root: &Dependencies) -> StateBox<AppState> {
        let counter1 = SimpleCounter::new(&root);
        let counter2 = SimpleCounter::new(&root);
        let counter3 = SimpleCounter::new(&root);

        let suma = {
            let counter1 = counter1.toComputed();
            let counter2 = counter2.toComputed();
            let counter3 = counter3.toComputed();

            root.from(move || {
                let value1 = *counter1.getValue().counter.getValue();
                let value2 = *counter2.getValue().counter.getValue();
                let value3 = *counter3.getValue().counter.getValue();

                value1 + value2 + value3
            })
        };

        StateBox::new(&root,AppState {
            value: root.newValue(33),
            at: root.newValue(999),
            counter1,
            counter2,
            counter3,
            suma
        })
    }

    pub fn increment(&self) {
        let rr = self.value.getValue();
        self.value.setValue(*rr + 1);
    }

    pub fn decrement(&self) {
        let rr = self.value.getValue();
        self.value.setValue(*rr - 1);
    }
}
