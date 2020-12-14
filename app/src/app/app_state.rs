use virtualdom::{
    computed::{
        Dependencies::Dependencies,
        Value::Value,
        Computed::Computed,
    }
};

use crate::simple_counter::simple_counter_state::SimpleCounter;
use crate::sudoku::state::Sudoku;

pub struct AppState {
    pub value: Value<u32>,
    pub at: Value<u32>,
    pub counter1: Computed<SimpleCounter>,
    pub counter2: Computed<SimpleCounter>,
    pub counter3: Computed<SimpleCounter>,
    pub counter4: Computed<SimpleCounter>,

    pub suma: Computed<u32>,

    pub sudoku: Computed<Sudoku>,
}

impl AppState {
    pub fn new(root: &Dependencies) -> Computed<AppState> {
        let counter1 = SimpleCounter::new(&root);
        let counter2 = SimpleCounter::new(&root);
        let counter3 = SimpleCounter::new(&root);
        let counter4 = SimpleCounter::new(&root);

        let suma = {
            // let counter1 = counter1.clone();
            // let value1 = *counter1.counter;

            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();

            root.from(move || {
                let value1 = *counter1.getValue().counter.getValue();
                let value2 = *counter2.getValue().counter.getValue();
                let value3 = *counter3.getValue().counter.getValue();

                value1 + value2 + value3
            })
        };

        root.newComputedFrom(AppState {
            value: root.newValue(33),
            at: root.newValue(999),
            counter1,
            counter2,
            counter3,
            counter4,
            suma,
            sudoku: Sudoku::new(root)
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

    // async fn cos() -> u32 {
    //     4
    // }
}
