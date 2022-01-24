use std::cmp::PartialEq;
use vertigo::{html, Computed, Driver, VDomElement, Value};
use vertigo::VDomComponent;

mod simple_counter;

#[derive(PartialEq)]
pub struct State {
    driver: Driver,
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
            driver: driver.clone(),
            counter1,
            counter2,
            counter3,
            counter4,
            sum
        };

        driver.bind_render(state, render)
    }
}

fn render_sum(state: &Computed<State>) -> VDomElement {
    let state = state.get_value();

    let sum = state.sum.get_value();

    html! {
        <div>
            {$ format!("sum = {}", sum) $}
        </div>
    }
}

pub fn render(state: &Computed<State>) -> VDomElement {
    let st = state.get_value();

    let view1 = simple_counter::State::component(&st.driver, &st.counter1);
    let view2 = simple_counter::State::component(&st.driver, &st.counter2);
    let view3 = simple_counter::State::component(&st.driver, &st.counter3);
    let view4 = simple_counter::State::component(&st.driver, &st.counter4);

    html! {
        <div>
            { view1 }
            { view2 }
            { view3 }
            { view4 }
            <component {render_sum} data={state} />
        </div>
    }
}
