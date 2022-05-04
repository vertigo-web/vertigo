use vertigo::{html, Computed, VDomElement, Value};
use vertigo::VDomComponent;

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
    pub fn component() -> VDomComponent {
        let counter1 = Value::new(0);
        let counter2 = Value::new(0);
        let counter3 = Value::new(0);
        let counter4 = Value::new(0);

        let sum = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            Computed::from(move || {
                let value1 = counter1.get();
                let value2 = counter2.get();
                let value3 = counter3.get();
                let value4 = counter4.get();

                value1 + value2 + value3 + value4
            })
        };

        let state = State {
            counter1,
            counter2,
            counter3,
            counter4,
            sum
        };

        render(state)
    }
}

fn render_sum(sum: &Computed<u32>) -> VDomElement {
    let sum = sum.get();

    html! {
        <div>
            {$ format!("sum = {}", sum) $}
        </div>
    }
}

pub fn render(state: State) -> VDomComponent {
    let view1 = simple_counter::State::component(&state.counter1);
    let view2 = simple_counter::State::component(&state.counter2);
    let view3 = simple_counter::State::component(&state.counter3);
    let view4 = simple_counter::State::component(&state.counter4);
    let view_sum = VDomComponent::from_ref(&state.sum, render_sum);

    VDomComponent::from(state, move |_: &State| {
        html! {
            <div>
                { view1.clone() }
                { view2.clone() }
                { view3.clone() }
                { view4.clone() }
                { view_sum.clone() }
            </div>
        }
    })
}
