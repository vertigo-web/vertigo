use vertigo::{Computed, Value, DomElement, dom};

mod simple_counter;

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
        let counter1 = Value::new(0);
        let counter2 = Value::new(0);
        let counter3 = Value::new(0);
        let counter4 = Value::new(0);

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

    pub fn render(&self) -> DomElement {
        render(self)
    }
}

fn render_sum(sum: &Computed<u32>) -> DomElement {
    let sum = sum.clone().map(|sum| { sum.to_string() });

    dom! {
        <div>
            "sum = "
            <text computed={sum} />
        </div>
    }
}

pub fn render(state: &State) -> DomElement {
    dom! {
        <div>
            { simple_counter::State::component(&state.counter1) }
            { simple_counter::State::component(&state.counter2) }
            { simple_counter::State::component(&state.counter3) }
            { simple_counter::State::component(&state.counter4) }
            { render_sum(&state.sum) }
        </div>
    }
}

