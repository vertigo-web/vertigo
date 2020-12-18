use virtualdom::{
    computed::{
        Dependencies,
        Value,
        Computed,
    }
};

use crate::simple_counter;
use crate::sudoku;

pub struct State {
    pub value: Value<u32>,
    pub at: Value<u32>,
    pub counter1: Computed<simple_counter::State>,
    pub counter2: Computed<simple_counter::State>,
    pub counter3: Computed<simple_counter::State>,
    pub counter4: Computed<simple_counter::State>,

    pub suma: Computed<u32>,

    pub sudoku: Computed<sudoku::Sudoku>,
}

impl State {
    pub fn new(root: &Dependencies) -> Computed<State> {
        let counter1 = simple_counter::State::new(&root);
        let counter2 = simple_counter::State::new(&root);
        let counter3 = simple_counter::State::new(&root);
        let counter4 = simple_counter::State::new(&root);

        let suma = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            root.from(move || {
                let value1 = *counter1.getValue().counter.getValue();
                let value2 = *counter2.getValue().counter.getValue();
                let value3 = *counter3.getValue().counter.getValue();
                let value4 = *counter4.getValue().counter.getValue();

                value1 + value2 + value3 + value4
            })
        };

        root.newComputedFrom(State {
            value: root.newValue(33),
            at: root.newValue(999),
            counter1,
            counter2,
            counter3,
            counter4,
            suma,
            sudoku: sudoku::Sudoku::new(root)
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
