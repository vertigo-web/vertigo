use vertigo::{Computed, Value, DomElement, dom};

mod simple_counter;
use simple_counter::SimpleCounter;

#[derive(Clone)]
pub struct State {
    counter1: Value<u32>,
    counter2: Value<u32>,
    counter3: Value<u32>,
    counter4: Value<u32>,
    sum: Computed<u32>,
}

impl State {
    pub fn new() -> State {
        let counter1 = Value::new(1);
        let counter2 = Value::new(2);
        let counter3 = Value::new(3);
        let counter4 = Value::new(4);

        let sum = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            Computed::from(move |context| {
                let value1 = counter1.get(context);
                let value2 = counter2.get(context);
                let value3 = counter3.get(context);
                let value4 = counter4.get(context);

                value1 + value2 + value3 + value4
            })
        };

        State {
            counter1,
            counter2,
            counter3,
            counter4,
            sum
        }
    }
}

struct Sum {
    sum: Computed<u32>,
}

impl Sum {
    pub fn mount(self) -> DomElement {
        dom! {
            <div>
                "sum = " {self.sum}
            </div>
        }
    }
}

pub struct CountersDemo {
    pub state: State,
}

impl CountersDemo {
    pub fn mount(&self) -> DomElement {
        dom! {
            <div>
                <SimpleCounter label="counter1 value" value={&self.state.counter1} />
                <SimpleCounter label="counter2 value" value={&self.state.counter2} />
                <SimpleCounter label="counter3 value" value={&self.state.counter3} />
                <SimpleCounter label="counter4 value" value={&self.state.counter4} />
                <Sum sum={&self.state.sum} />
            </div>
        }
    }
}
