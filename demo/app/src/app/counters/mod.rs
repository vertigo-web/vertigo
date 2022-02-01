use vertigo::{html, Computed, Driver, VDomElement, Value};
use vertigo::VDomComponent;

mod simple_counter;

pub struct State {
    counter1: Value<u32>,
    counter2: Value<u32>,
    counter3: Value<u32>,
    counter4: Value<u32>,
    sum: Computed<u32>,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let counter1 = driver.new_value(0);
        let counter2 = driver.new_value(0);
        let counter3 = driver.new_value(0);
        let counter4 = driver.new_value(0);

        let sum = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            driver.from(move || {
                let value1 = *counter1.get_value();
                let value2 = *counter2.get_value();
                let value3 = *counter3.get_value();
                let value4 = *counter4.get_value();

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
    let sum = sum.get_value();

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
    let view_sum = VDomComponent::new(state.sum.clone(), render_sum);

    VDomComponent::new(state, move |_: &State| {
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
